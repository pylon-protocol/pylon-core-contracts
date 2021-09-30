use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{HumanAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub this: HumanAddr,
    pub owner: HumanAddr,
    pub beneficiary: HumanAddr,
    pub base_price: Decimal256,
    pub min_user_cap: Uint256,
    pub max_user_cap: Uint256,
    pub staking_contract: HumanAddr,
    pub min_stake_amount: Uint256,
    pub max_stake_amount: Uint256,
    pub additional_cap_per_token: Decimal256,
    pub total_sale_amount: Uint256,
    pub start: u64,
    pub finish: u64,
}

pub fn store(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

pub fn read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, KEY_CONFIG)
}
