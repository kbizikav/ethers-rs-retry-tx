use env_logger::Builder;
use ethers::types::U256;
use log::LevelFilter;

use crate::{erc20_contract::ERC20Contract, utils::get_address};

#[tokio::test]
async fn deploy_erc20_contract() -> anyhow::Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();

    dotenv::dotenv().ok();
    let env = envy::from_env::<crate::env::EnvVar>()?;
    let address = get_address(env.chain_id, env.private_key);
    let contract =
        ERC20Contract::deploy(&env.rpc_url, env.chain_id, env.private_key, address).await?;
    println!(
        "Deployed ERC20 contract at address: {:?}",
        contract.address()
    );
    Ok(())
}

#[tokio::test]
async fn approve_token() -> anyhow::Result<()> {
    Builder::new().filter_level(LevelFilter::Info).init();

    dotenv::dotenv().ok();
    let env = envy::from_env::<crate::env::EnvVar>()?;
    let contract_address = std::env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set");

    let contract = ERC20Contract::new(&env.rpc_url, env.chain_id, contract_address.parse()?);

    contract
        .approve(env.private_key, contract.address, U256::zero())
        .await?;

    Ok(())
}
