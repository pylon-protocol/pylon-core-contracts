use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: String,
    pub input_denom: String,
    pub output_token: String,
    pub exchange_rate: Decimal256,
}

pub fn store<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    singleton(storage, KEY_CONFIG).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<Config> {
    singleton_read(storage, KEY_CONFIG).load()
}
