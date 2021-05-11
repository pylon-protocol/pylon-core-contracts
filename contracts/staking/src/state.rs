use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, StdResult, Storage, Uint128};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_REWARD: &[u8] = b"reward";
pub static PREFIX_USER: &[u8] = b"user";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub dp_token: CanonicalAddr,
    pub reward_token: CanonicalAddr,
    pub start_time: u64,
    pub finish_time: u64,
    pub open_deposit: bool,
    pub open_withdraw: bool,
    pub open_claim: bool,
    pub reward_rate: Decimal256,
}

pub fn store_config<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config<S: Storage>(storage: &S) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reward {
    pub total_deposit: Uint128,
    pub last_update_time: u64,
    pub reward_per_token_stored: Decimal256,
}

pub fn store_reward<S: Storage>(storage: &mut S, data: &Reward) -> StdResult<()> {
    Singleton::new(storage, KEY_REWARD).save(data)
}

pub fn read_reward<S: Storage>(storage: &S) -> StdResult<Reward> {
    ReadonlySingleton::new(storage, KEY_REWARD).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub amount: Uint128,
    pub reward: Uint128,
    pub reward_per_token_paid: Decimal256,
}

pub fn read_user<S: ReadonlyStorage>(storage: &S, owner: &CanonicalAddr) -> StdResult<User> {
    match ReadonlyBucket::new(PREFIX_USER, storage).may_load(owner.as_slice())? {
        Some(user) => Ok(user),
        None => Ok(User {
            amount: Uint128::zero(),
            reward: Uint128::zero(),
            reward_per_token_paid: Decimal256::zero(),
        }),
    }
}

pub fn store_user<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    user: &User,
) -> StdResult<()> {
    Bucket::new(PREFIX_USER, storage).save(owner.as_slice(), user)
}

pub fn remove_user<S: Storage>(storage: &mut S, owner: &CanonicalAddr) {
    Bucket::<S, User>::new(PREFIX_USER, storage).remove(owner.as_slice())
}
