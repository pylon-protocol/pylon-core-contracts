use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static STATE_KEY: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub next_pool_id: u64,
}

pub fn store<S: Storage>(storage: &mut S, data: &State) -> StdResult<()> {
    Singleton::new(storage, STATE_KEY).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<State> {
    ReadonlySingleton::new(storage, STATE_KEY).load()
}
