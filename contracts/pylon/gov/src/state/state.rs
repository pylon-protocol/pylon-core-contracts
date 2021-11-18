use cosmwasm_std::{StdResult, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static KEY_STATE: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
    pub total_airdrop_count: u64,
    pub airdrop_update_candidates: Vec<u64>,
}

impl State {
    pub fn load(storage: &dyn Storage) -> StdResult<State> {
        singleton_read(storage, KEY_STATE).load()
    }

    pub fn save(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
        singleton(storage, KEY_STATE).save(state)
    }
}
