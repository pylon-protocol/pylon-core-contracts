use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub this: CanonicalAddr,
    pub owner: CanonicalAddr,
    pub beneficiary: CanonicalAddr,
    pub fee_collector: CanonicalAddr,
    pub exchange_rate_feeder: CanonicalAddr,
    pub moneymarket: CanonicalAddr,
    pub atoken: CanonicalAddr,
    pub stable_denom: String,
    pub dp_token: CanonicalAddr,
}

pub fn store<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    Singleton::new(storage, CONFIG_KEY).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<Config> {
    ReadonlySingleton::new(storage, CONFIG_KEY).load()
}
