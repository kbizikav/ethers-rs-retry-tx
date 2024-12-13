use std::sync::Arc;

use ethers::{
    core::k256::{ecdsa::SigningKey, SecretKey},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{Signer as _, Wallet},
    types::{Address, BlockNumber, H256, U256},
};

use crate::{error::BlockchainError, retry::with_retry};

async fn get_provider(rpc_url: &str) -> Result<Provider<Http>, BlockchainError> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| BlockchainError::ParseError(format!("Failed to parse rpc url: {}", e)))?;
    Ok(provider)
}

pub async fn get_client(rpc_url: &str) -> Result<Arc<Provider<Http>>, BlockchainError> {
    Ok(Arc::new(get_provider(rpc_url).await?))
}

pub fn get_wallet(chain_id: u64, private_key: H256) -> Wallet<SigningKey> {
    let key = SecretKey::from_bytes(private_key.as_bytes().into()).unwrap();
    Wallet::from(key).with_chain_id(chain_id)
}

pub fn get_address(chain_id: u64, private_key: H256) -> Address {
    get_wallet(chain_id, private_key).address()
}

pub async fn get_gas_price(rpc_url: &str) -> Result<U256, BlockchainError> {
    let client = get_client(rpc_url).await?;
    let gas_price = with_retry(|| async { client.get_gas_price().await })
        .await
        .map_err(|_| BlockchainError::RPCError("failed to get gas price".to_string()))?;
    Ok(gas_price)
}

pub async fn get_base_fee(rpc_url: &str) -> Result<U256, BlockchainError> {
    let client = get_client(rpc_url).await?;
    let latest_block = with_retry(|| async { client.get_block(BlockNumber::Latest).await })
        .await
        .map_err(|_| BlockchainError::RPCError("failed to get latest block".to_string()))?
        .expect("latest block not found");
    let base_fee = latest_block
        .base_fee_per_gas
        .ok_or(BlockchainError::BlockBaseFeeNotFound)?;
    Ok(base_fee)
}

pub async fn estimate_eip1559_fees(rpc_url: &str) -> Result<(U256, U256), BlockchainError> {
    let client = get_client(rpc_url).await?;
    let (max_fee_per_gas, max_priority_fee_per_gas) =
        with_retry(|| async { client.estimate_eip1559_fees(None).await })
            .await
            .map_err(|_| {
                BlockchainError::RPCError("failed to get max priority fee per gas".to_string())
            })?;
    Ok((max_fee_per_gas, max_priority_fee_per_gas))
}

pub async fn get_client_with_signer(
    rpc_url: &str,
    chain_id: u64,
    private_key: H256,
) -> Result<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>, BlockchainError> {
    let provider = get_provider(rpc_url).await?;
    let wallet = get_wallet(chain_id, private_key);
    let client = SignerMiddleware::new(provider, wallet);
    Ok(client)
}

pub async fn get_latest_block_number(rpc_url: &str) -> Result<u64, BlockchainError> {
    let client = get_client(rpc_url).await?;
    let block_number = with_retry(|| async { client.get_block_number().await })
        .await
        .map_err(|_| BlockchainError::RPCError("failed to get block number".to_string()))?;
    Ok(block_number.as_u64())
}

pub async fn get_eth_balance(rpc_url: &str, address: Address) -> Result<U256, BlockchainError> {
    let client = get_client(rpc_url).await?;
    let balance = with_retry(|| async { client.get_balance(address, None).await })
        .await
        .map_err(|_| BlockchainError::RPCError("failed to get block number".to_string()))?;
    Ok(balance)
}

pub async fn get_transaction(
    rpc_url: &str,
    tx_hash: H256,
) -> Result<Option<ethers::types::Transaction>, BlockchainError> {
    let client = get_client(rpc_url).await?;
    let tx = with_retry(|| async { client.get_transaction(tx_hash).await })
        .await
        .map_err(|_| BlockchainError::RPCError("failed to get transaction".to_string()))?;
    Ok(tx)
}
