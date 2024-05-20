use std::{collections::HashMap, sync::Arc};

use clap::Parser;
use node::Node;
use threshold_crypto::SecretKey;
use tokio::sync::Mutex;

mod net;
mod node;
mod transactions;
mod wallet;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  #[clap(short, long, default_value = "0")]
  node_id: usize,

  #[clap(short, long)]
  peers: Vec<String>,

  #[clap(short, long, default_value = "127.0.0.1:8080")]
  address: String,
}

#[tokio::main]
async fn main() {
  let args = Args::parse();

  let node_id = args.node_id;
  let address = args.address.clone();
  let initial_peers = args.peers.clone();
  let secret_key = SecretKey::random();
  let public_key = secret_key.public_key();
  let result_store = Arc::new(Mutex::new(Vec::new()));
  let sig_shares_store = Arc::new(Mutex::new(Vec::new()));
  let wallets = Arc::new(Mutex::new(HashMap::new()));

  let node = Arc::new(Node::new(
    node_id,
    address.clone(),
    secret_key,
    public_key,
    Arc::clone(&result_store),
    Arc::clone(&sig_shares_store),
    Arc::clone(&wallets),
  ));

  node.run(initial_peers).await;
}
