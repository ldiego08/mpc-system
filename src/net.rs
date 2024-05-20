use serde::{Deserialize, Serialize};
use threshold_crypto::PublicKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PeerRegistrationRequest {
  pub node_id: usize,
  pub public_key: PublicKey,
  pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PeerRegistrationResponse {
  pub node_id: usize,
  pub public_key: PublicKey,
  pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WalletCreationRequest {
  pub initial_balance: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WalletCreationResponse {
  pub wallet_id: String,
  pub initial_balance: i32,
}
