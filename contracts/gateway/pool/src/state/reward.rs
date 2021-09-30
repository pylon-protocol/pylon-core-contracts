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

pub fn store(storage: &mut dyn Storage, data: &Reward) -> StdResult<()> {
    Singleton::new(storage, KEY_REWARD).save(data)
}

pub fn read(storage: &dyn Storage) -> StdResult<Reward> {
    ReadonlySingleton::new(storage, KEY_REWARD).load()
}
