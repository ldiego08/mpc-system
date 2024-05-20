use serde::{Deserialize, Serialize};
use threshold_crypto::{Signature, SignatureShare};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Transaction {
  pub from: String,
  pub to: String,
  pub amount: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SignedTransaction {
  pub transaction: Transaction,
  pub sender_id: usize,
  pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransactionResult {
  pub node_id: usize,
  pub success: bool,
  pub transaction: Option<SignedTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransactionVerificationRequest {
  pub result: TransactionResult,
  pub signature_share: SignatureShare,
}
