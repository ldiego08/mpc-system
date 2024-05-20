use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Wallet {
  pub id: String,
  pub balance: i32,
}
