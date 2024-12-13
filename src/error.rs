use ethers::types::H256;

#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("RPC error: {0}")]
    RPCError(String),

    #[error("Decode call data error: {0}")]
    DecodeCallDataError(String),

    #[error("Token not found")]
    TokenNotFound,

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Failed to get transaction receipt: {0:?} with error: {1}")]
    FailedToGetTransactionReceipt(H256, String),

    #[error("Block not found: {0}")]
    BlockNotFound(u64),

    #[error("Block base fee not found")]
    BlockBaseFeeNotFound,

    #[error("Transaction not found: {0:?}")]
    TxNotFound(H256),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Max tx retries reached")]
    MaxTxRetriesReached,
}
