use ethers::types::H256;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct EnvVar {
    pub rpc_url: String,
    pub chain_id: u64,
    pub private_key: H256,
}
