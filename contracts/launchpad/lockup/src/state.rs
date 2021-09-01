use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Order, Querier, ReadonlyStorage, StdResult, Storage,
};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_REWARD: &[u8] = b"reward";
pub static PREFIX_USER: &[u8] = b"user";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub share_token: CanonicalAddr,
    pub reward_token: CanonicalAddr,
    pub start_time: u64,
    pub cliff_time: u64,
    pub finish_time: u64,
    pub temp_withdraw_start_time: u64,
    pub temp_withdraw_finish_time: u64,
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
    pub total_deposit: Uint256,
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
    pub amount: Uint256,
    pub reward: Uint256,
    pub reward_per_token_paid: Decimal256,
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

pub fn read_user<S: ReadonlyStorage>(storage: &S, owner: &CanonicalAddr) -> StdResult<User> {
    match ReadonlyBucket::new(PREFIX_USER, storage).may_load(owner.as_slice())? {
        Some(user) => Ok(user),
        None => Ok(User {
            amount: Uint256::zero(),
            reward: Uint256::zero(),
            reward_per_token_paid: Decimal256::zero(),
        }),
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn batch_read_user<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
) -> StdResult<Vec<(HumanAddr, User)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);

    ReadonlyBucket::new(PREFIX_USER, &deps.storage)
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|elem: StdResult<(Vec<u8>, User)>| {
            let (k, v) = elem.unwrap();
            let user = deps.api.human_address(&CanonicalAddr::from(k))?;
            Ok((user, v))
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<CanonicalAddr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_slice().to_vec();
        v.push(1);
        v
    })
}
