use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_VPOOL: &[u8] = b"vpool";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VirtualPool {
    pub x_denom: String,
    pub y_addr: CanonicalAddr,
    pub liq_x: Uint256,
    pub liq_y: Uint256,
}

pub fn store(storage: &mut dyn Storage) -> Singleton<VirtualPool> {
    singleton(storage, KEY_VPOOL)
}

pub fn read(storage: &dyn Storage) -> ReadonlySingleton<VirtualPool> {
    singleton_read(storage, KEY_VPOOL)
}
