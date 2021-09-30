use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: String,
    pub moneymarket: String,
    pub input_denom: String,
    pub yield_token: String,
}

pub fn store<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    singleton(storage, CONFIG_KEY).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<Config> {
    singleton_read(storage, CONFIG_KEY).load()
}
