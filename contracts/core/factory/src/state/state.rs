use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};

pub static STATE_KEY: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub next_pool_id: Uint256,
}

pub fn store<S: Storage>(storage: &mut S, data: &State) -> StdResult<()> {
    Singleton::new(storage, STATE_KEY).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<State> {
    ReadonlySingleton::new(storage, STATE_KEY).load()
}
