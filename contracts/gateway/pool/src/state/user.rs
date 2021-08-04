use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static PREFIX_USER: &[u8] = b"user";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub next_withdrawal_index: u64,
    pub claimed_withdrawal_index: u64,
    pub pending_withdrawal_amount: Uint256,
    pub amount: Uint256,
    pub reward: Uint256,
    pub reward_per_token_paid: Decimal256,
}

pub fn read<S: ReadonlyStorage>(storage: &S, owner: &CanonicalAddr) -> StdResult<User> {
    match ReadonlyBucket::new(PREFIX_USER, storage).may_load(owner.as_slice())? {
        Some(user) => Ok(user),
        None => Ok(User {
            next_withdrawal_index: 0,
            claimed_withdrawal_index: 0,
            pending_withdrawal_amount: Uint256::zero(),
            amount: Uint256::zero(),
            reward: Uint256::zero(),
            reward_per_token_paid: Decimal256::zero(),
        }),
    }
}

pub fn store<S: Storage>(storage: &mut S, owner: &CanonicalAddr, user: &User) -> StdResult<()> {
    Bucket::new(PREFIX_USER, storage).save(owner.as_slice(), user)
}

pub fn remove<S: Storage>(storage: &mut S, owner: &CanonicalAddr) {
    Bucket::<S, User>::new(PREFIX_USER, storage).remove(owner.as_slice())
}
