use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
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

pub fn store<S: Storage>(storage: &mut S, data: &VirtualPool) -> StdResult<()> {
    Singleton::new(storage, KEY_VPOOL).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<VirtualPool> {
    ReadonlySingleton::new(storage, KEY_VPOOL).load()
}
