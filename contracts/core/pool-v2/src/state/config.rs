use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub id: u64,
    pub name: String,
    pub factory: String,
    pub beneficiary: String,
    pub yield_adapter: String,
    pub input_denom: String,
    pub yield_token: String,
    pub dp_token: String,
}

pub fn store(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    singleton(storage, CONFIG_KEY).save(data)
}

pub fn read(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, CONFIG_KEY).load()
}
