use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Order, Querier, ReadonlyStorage, StdResult, Storage,
};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static PREFIX_USER: &[u8] = b"user_state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserState {
    pub processed: bool,
}

pub fn store<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    user: &UserState,
) -> StdResult<()> {
    Bucket::new(PREFIX_USER, storage).save(owner.as_slice(), user)
}

pub fn remove<S: Storage>(storage: &mut S, owner: &CanonicalAddr) {
    Bucket::<S, UserState>::new(PREFIX_USER, storage).remove(owner.as_slice())
}

pub fn read<S: ReadonlyStorage>(storage: &S, owner: &CanonicalAddr) -> StdResult<UserState> {
    match ReadonlyBucket::new(PREFIX_USER, storage).may_load(owner.as_slice())? {
        Some(user) => Ok(user),
        None => Ok(UserState { processed: false }),
    }
}
