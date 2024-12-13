use std::str::FromStr as _;

use ethers::types::H256;

use crate::{erc20_contract::ERC20Contract, utils::get_address};

#[tokio::test]
async fn deploy_erc20_contract() -> anyhow::Result<()> {
    let rpc_url = "http://localhost:8545";
    let chain_id = 31337;

    let private_key =
        H256::from_str("0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba")?;
    let address = get_address(chain_id, private_key);

    let contract = ERC20Contract::deploy(rpc_url, chain_id, private_key, address).await?;

    let balance = contract.balance_of(address).await?;
    println!("Balance of {}: {}", address, balance);
    Ok(())
}
