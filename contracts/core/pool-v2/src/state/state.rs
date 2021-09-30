use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{singleton, singleton_read};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static STATE_KEY: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub accumulated_reward: Uint256,
    pub accumulated_fee: Uint256,
}

pub fn store(storage: &mut dyn Storage, data: &State) -> StdResult<()> {
    singleton(storage, STATE_KEY).save(data)
}

pub fn read(storage: &dyn Storage) -> StdResult<State> {
    singleton_read(storage, STATE_KEY).load()
}
