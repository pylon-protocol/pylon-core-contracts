use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub start_time: u64,
    pub finish_time: u64,

    pub depositable: bool,
    pub withdrawable: bool,
    pub cliff_period: u64,
    pub vesting_period: u64,
    pub unbonding_period: u64,
    pub reward_rate: Decimal256,

    pub staking_token: CanonicalAddr,
    pub reward_token: CanonicalAddr,
}

pub fn store<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}
