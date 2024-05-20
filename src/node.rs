use std::{collections::HashMap, sync::Arc};

use reqwest::Client;
use serde::Serialize;
use threshold_crypto::{PublicKey, SecretKey, SignatureShare};
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::Filter;

use crate::{
  net::{PeerRegistrationRequest, WalletCreationRequest, WalletCreationResponse},
  transactions::{
    SignedTransaction, Transaction, TransactionResult, TransactionVerificationRequest,
  },
  wallet::Wallet,
};

pub(crate) type NodePeers = Arc<Mutex<HashMap<usize, (PublicKey, String)>>>;

pub(crate) type NodeTransactionResultStore = Arc<Mutex<Vec<TransactionResult>>>;

pub(crate) type NodeSignatureShareStore = Arc<Mutex<Vec<SignatureShare>>>;

pub(crate) type NodeWallets = Arc<Mutex<HashMap<String, Wallet>>>;

#[derive(Clone)]
pub(crate) struct Node {
  id: usize,
  secret_key: Arc<SecretKey>,
  public_key: Arc<PublicKey>,
  address: String,
  peers: NodePeers,
  transaction_result_store: NodeTransactionResultStore,
  signature_shares_store: NodeSignatureShareStore,
  wallets: NodeWallets,
}

impl Node {
  pub fn new(
    id: usize,
    address: String,
    secret_key: SecretKey,
    public_key: PublicKey,
    transaction_result_store: NodeTransactionResultStore,
    signature_shares_store: NodeSignatureShareStore,
    wallets: NodeWallets,
  ) -> Self {
    let peers = Arc::new(Mutex::new(HashMap::new()));

    Node {
      id,
      peers,
      address,
      wallets,
      secret_key: Arc::new(secret_key),
      public_key: Arc::new(public_key),
      signature_shares_store,
      transaction_result_store,
    }
  }

  pub async fn run(&self, initial_peers: Vec<String>) {
    self.register_with_peers(initial_peers).await;

    let node = self.clone();
    let register_peer_route = warp::post()
      .and(warp::path("register"))
      .and(warp::body::json())
      .and_then(move |request: PeerRegistrationRequest| {
        let node = node.clone();
        async move {
          node.handle_incoming_peer_registration(request).await;
          Ok::<_, warp::Rejection>(warp::reply::json(&"Peer registered"))
        }
      });

    let node = self.clone();
    let peers_route = warp::get().and(warp::path("peers")).and_then(move || {
      let node = node.clone();
      async move {
        let peers = node.get_peers().await;
        Ok::<_, warp::Rejection>(warp::reply::json(&peers))
      }
    });

    let node = self.clone();
    let transaction_route = warp::post()
      .and(warp::path("transaction"))
      .and(warp::body::json())
      .and_then(move |request: SignedTransaction| {
        let node = node.clone();
        async move {
          node.handle_incoming_transaction(request.transaction).await;
          Ok::<_, warp::Rejection>(warp::reply::json(&"Transaction processed"))
        }
      });

    let node = self.clone();
    let transaction_verification_route = warp::post()
      .and(warp::path("verification"))
      .and(warp::body::json())
      .and_then(move |request: TransactionVerificationRequest| {
        let node = node.clone();
        async move {
          node.handle_incoming_transaction_verification(request).await;
          Ok::<_, warp::Rejection>(warp::reply::json(&"Verification processed"))
        }
      });

    let node = self.clone();
    let wallet_creation_route = warp::post()
      .and(warp::path("wallet_creation"))
      .and(warp::body::json())
      .and_then(move |request: WalletCreationRequest| {
        let node = node.clone();
        async move {
          node.handle_incoming_wallet_creation(request).await;
          Ok::<_, warp::Rejection>(warp::reply::json(&"Wallet created"))
        }
      });

    let routes = register_peer_route
      .or(peers_route)
      .or(transaction_route)
      .or(transaction_verification_route)
      .or(wallet_creation_route);

    warp::serve(routes)
      .run((
        [127, 0, 0, 1],
        self.address.split(':').nth(1).unwrap().parse().unwrap(),
      ))
      .await;
  }

  pub async fn register_with_peers(&self, initial_peers: Vec<String>) {
    for address in initial_peers {
      let client = Client::new();
      let response = client
        .post(format!("http://{}/register", address))
        .json(&PeerRegistrationRequest {
          node_id: self.id,
          public_key: (*self.public_key).clone(),
          address: self.address.clone(),
        })
        .send()
        .await;
      if let Ok(response) = response {
        if let Ok(peer_registration) = response.json::<PeerRegistrationRequest>().await {
          self.peers.lock().await.insert(
            peer_registration.node_id,
            (
              peer_registration.public_key,
              peer_registration.address.clone(),
            ),
          );
        }
      }
    }
  }

  pub async fn get_peers(&self) -> HashMap<usize, (PublicKey, String)> {
    self.peers.lock().await.clone()
  }

  async fn broadcast_to_peers<T: Serialize>(&self, path: &str, message: &T) {
    let peers = self.peers.lock().await;
    for (_, (_, address)) in peers.iter() {
      let client = Client::new();
      let _ = client
        .post(format!("http://{}/{}", address, path))
        .json(message)
        .send()
        .await;
    }
  }

  pub async fn handle_incoming_transaction_verification(
    &self,
    verification_request: TransactionVerificationRequest,
  ) {
    if self.public_key.verify(
      &verification_request.signature_share.0,
      &serde_json::to_vec(&verification_request.result).unwrap(),
    ) {
      let mut store = self.transaction_result_store.lock().await;
      let mut sig_shares = self.signature_shares_store.lock().await;

      store.push(verification_request.result);
      sig_shares.push(verification_request.signature_share);
    }
  }

  async fn handle_incoming_transaction(&self, transaction: Transaction) -> TransactionResult {
    let signed_transaction = self.sign_transaction(transaction.clone());
    let success = self.process_transaction(&signed_transaction).await;

    let result = TransactionResult {
      node_id: self.id,
      success,
      transaction: if success {
        Some(signed_transaction.clone())
      } else {
        None
      },
    };

    let signature_share =
      SignatureShare(self.secret_key.sign(&serde_json::to_vec(&result).unwrap()));

    let verification_request = TransactionVerificationRequest {
      result: result.clone(),
      signature_share,
    };

    self
      .broadcast_to_peers("verification", &verification_request)
      .await;

    if success {
      self
        .broadcast_to_peers("transaction", &signed_transaction)
        .await;
    }

    result
  }

  fn sign_transaction(&self, transaction: Transaction) -> SignedTransaction {
    let signature = self
      .secret_key
      .sign(&serde_json::to_vec(&transaction).unwrap());

    SignedTransaction {
      transaction,
      sender_id: self.id,
      signature,
    }
  }

  async fn process_transaction(&self, signed_transaction: &SignedTransaction) -> bool {
    let mut wallets = self.wallets.lock().await;

    if let Some((public_key, _)) = self.peers.lock().await.get(&signed_transaction.sender_id) {
      if public_key.verify(
        &signed_transaction.signature,
        &serde_json::to_vec(&signed_transaction.transaction).unwrap(),
      ) {
        let transaction = &signed_transaction.transaction;
        let sufficient_balance = if let Some(sender_wallet) = wallets.get_mut(&transaction.from) {
          if sender_wallet.balance >= transaction.amount {
            sender_wallet.balance -= transaction.amount;
            true
          } else {
            false
          }
        } else {
          false
        };

        if sufficient_balance {
          if let Some(receiver_wallet) = wallets.get_mut(&transaction.to) {
            receiver_wallet.balance += transaction.amount;
            return true;
          }
        }
      }
    }
    false
  }

  pub async fn handle_incoming_wallet_creation(&self, wallet_creation: WalletCreationRequest) {
    let wallet_id = Uuid::new_v4().to_string();
    let mut wallets = self.wallets.lock().await;

    wallets.insert(
      wallet_id.clone(),
      Wallet {
        id: wallet_id.clone(),
        balance: wallet_creation.initial_balance,
      },
    );
    drop(wallets);

    let response = WalletCreationResponse {
      wallet_id,
      initial_balance: wallet_creation.initial_balance,
    };

    self.broadcast_to_peers("wallet_creation", &response).await;

    println!("Wallet created: {:?}", response);
  }

  pub async fn handle_incoming_peer_registration(
    &self,
    peer_registration: PeerRegistrationRequest,
  ) {
    self.peers.lock().await.insert(
      peer_registration.node_id,
      (peer_registration.public_key, peer_registration.address),
    );

    println!(
      "Node {} registered peer {}",
      self.id, peer_registration.node_id
    );
  }
}
