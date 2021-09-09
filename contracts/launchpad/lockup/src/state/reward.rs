use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_REWARD: &[u8] = b"reward";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reward {
    pub total_deposit: Uint256,
    pub last_update_time: u64,
    pub reward_per_token_stored: Decimal256,
}

pub fn store<S: Storage>(storage: &mut S, data: &Reward) -> StdResult<()> {
    Singleton::new(storage, KEY_REWARD).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<Reward> {
    ReadonlySingleton::new(storage, KEY_REWARD).load()
}
