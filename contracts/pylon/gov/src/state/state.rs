use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static KEY_STATE: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub contract_addr: CanonicalAddr,
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

pub fn store<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, KEY_STATE)
}

pub fn read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, KEY_STATE)
}
