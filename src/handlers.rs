use std::time::Duration;

use ethers::{
    abi::Detokenize,
    core::k256::ecdsa::SigningKey,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::Wallet,
    types::{Transaction, H256, U256},
};
use tokio::time::sleep;

use crate::{
    error::BlockchainError,
    retry::with_retry,
    utils::{estimate_eip1559_fees, get_base_fee},
};

const MAX_GAS_BUMP_ATTEMPTS: u32 = 3;
const WAIT_TIME: Duration = Duration::from_secs(60);
const GAS_BUMP_PERCENTAGE: u64 = 10;

pub async fn handle_contract_call<S: ToString, O: Detokenize>(
    client: &SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    tx: &mut ethers::contract::builders::ContractCall<
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        O,
    >,
    tx_name: S,
) -> Result<H256, BlockchainError> {
    set_gas_price(client.provider().url().as_str(), tx).await?;
    let result = tx.send().await;
    match result {
        Ok(tx) => {
            let pending_tx = tx;
            let tx_hash = pending_tx.tx_hash();
            log::info!("{} tx hash: {:?}", tx_name.to_string(), tx_hash);
            let tx: Transaction = with_retry(|| async { client.get_transaction(tx_hash).await })
                .await
                .map_err(|e| BlockchainError::RPCError(e.to_string()))?
                .ok_or(BlockchainError::TxNotFound(tx_hash))?;
            send_tx_with_eip1559_gas_bump(
                client,
                tx,
                tx_name,
                MAX_GAS_BUMP_ATTEMPTS,
                WAIT_TIME,
                GAS_BUMP_PERCENTAGE,
            )
            .await
        }
        Err(e) => {
            let error_message = e.to_string();
            return Err(BlockchainError::TransactionError(format!(
                "{} failed with error: {:?}",
                tx_name.to_string(),
                error_message
            )));
        }
    }
}

async fn send_tx_with_eip1559_gas_bump<S: ToString>(
    client: &SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    tx: Transaction,
    tx_name: S,
    max_attempts: u32,
    wait_time: Duration,
    gas_bump_percentage: u64,
) -> Result<H256, BlockchainError> {
    let mut current_tx = tx.clone();
    let mut attempt = 0;
    while attempt < max_attempts {
        let base_fee = get_base_fee(client.provider().url().as_str()).await?;
        let priority_fee = tx
            .max_priority_fee_per_gas
            .unwrap_or(U256::from(2_000_000_000));
        let new_priority_fee = priority_fee * (100 + gas_bump_percentage) / 100;
        current_tx.max_priority_fee_per_gas = Some(new_priority_fee);
        current_tx.max_fee_per_gas = Some(base_fee * 2 + new_priority_fee);
        log::info!(
            "Bumping gas for {} tx attempt: {} with new max_fee_per_gas: {:?}, new max_priority_fee_per_gas: {:?}",
            tx_name.to_string(),
            attempt,
            current_tx.max_fee_per_gas,
            current_tx.max_priority_fee_per_gas
        );
        let pending_tx = client
            .send_transaction(&current_tx, None)
            .await
            .map_err(|e| BlockchainError::RPCError(e.to_string()))?;
        sleep(wait_time).await;
        match client
            .get_transaction_receipt(pending_tx.tx_hash())
            .await
            .map_err(|e| BlockchainError::RPCError(e.to_string()))?
        {
            Some(tx_receipt) => {
                if tx_receipt.status.unwrap() != 1.into() {
                    return Err(BlockchainError::TransactionFailed(format!(
                        "{} failed with tx hash: {:?}",
                        tx_name.to_string(),
                        tx_receipt.transaction_hash
                    )));
                }
                return Ok(tx_receipt.transaction_hash);
            }
            None => attempt += 1,
        }
    }
    return Err(BlockchainError::MaxTxRetriesReached);
}

async fn set_gas_price<O>(
    rpc_url: &str,
    tx: &mut ethers::contract::builders::ContractCall<
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        O,
    >,
) -> Result<(), BlockchainError> {
    let (max_fee_per_gas, max_priority_fee_per_gas) = estimate_eip1559_fees(rpc_url).await?;
    log::info!(
        "max_fee_per_gas: {:?}, max_priority_fee_per_gas: {:?}",
        max_fee_per_gas,
        max_priority_fee_per_gas
    );
    let inner_tx = tx.tx.as_eip1559_mut().expect("EIP-1559 tx expected");
    *inner_tx = inner_tx
        .clone()
        .max_priority_fee_per_gas(max_priority_fee_per_gas)
        .max_fee_per_gas(max_fee_per_gas);
    Ok(())
}
