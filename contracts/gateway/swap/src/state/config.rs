use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::Storage;
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use pylon_gateway::swap_msg::Strategy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: String,
    pub beneficiary: String,
    pub price: Decimal256,
    pub start: u64,
    pub finish: u64,
    pub cap_strategy: Option<String>,
    pub distribution_strategy: Vec<Strategy>,
    pub whitelist_enabled: bool,
    pub swap_pool_size: Uint256,
}

pub fn store(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

pub fn read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, KEY_CONFIG)
}
