use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: String,
    pub pool_code_id: u64,
    pub token_code_id: u64,
    pub fee_rate: Decimal256,
    pub fee_collector: String,
}

pub fn store(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    singleton(storage, CONFIG_KEY).save(data)
}

pub fn read(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, CONFIG_KEY).load()
}
