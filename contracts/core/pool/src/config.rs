use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub this: CanonicalAddr,
    pub owner: CanonicalAddr,
    pub beneficiary: CanonicalAddr,
    pub fee_collector: CanonicalAddr,
    pub moneymarket: CanonicalAddr,
    pub atoken: CanonicalAddr,
    pub stable_denom: String,
    pub dp_token: CanonicalAddr,
}

pub fn store(storage: &mut dyn Storage, data: &Config) -> StdResult<()> {
    singleton(storage, CONFIG_KEY).save(data)
}

pub fn read(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read(storage, CONFIG_KEY).load()
}
