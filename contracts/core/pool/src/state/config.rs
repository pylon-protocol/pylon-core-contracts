use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub id: u64,
    pub name: String,
    pub this: CanonicalAddr,
    pub factory: CanonicalAddr,
    pub beneficiary: CanonicalAddr,
    pub fee_collector: CanonicalAddr,
    pub yield_adapter: CanonicalAddr,
    pub input_denom: String,
    pub yield_token: CanonicalAddr,
    pub dp_token: CanonicalAddr,
}

pub fn store<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    Singleton::new(storage, CONFIG_KEY).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<Config> {
    ReadonlySingleton::new(storage, CONFIG_KEY).load()
}
