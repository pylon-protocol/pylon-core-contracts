use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::*;
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static PREFIX_USER: &[u8] = b"user";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub amount: Uint256,
    pub reward: Uint256,
    pub reward_per_token_paid: Decimal256,
}

pub fn store(storage: &mut dyn Storage, owner: &CanonicalAddr, user: &User) -> StdResult<()> {
    let mut user_bucket: Bucket<User> = bucket(storage, PREFIX_USER);

    user_bucket.save(owner.as_slice(), user)
}

pub fn remove(storage: &mut dyn Storage, owner: &CanonicalAddr) {
    let mut user_bucket: Bucket<User> = bucket(storage, PREFIX_USER);

    user_bucket.remove(owner.as_slice())
}

pub fn read(storage: &dyn Storage, owner: &CanonicalAddr) -> StdResult<User> {
    let user_bucket: ReadonlyBucket<User> = bucket_read(storage, PREFIX_USER);

    match user_bucket.may_load(owner.as_slice())? {
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
pub fn batch_read(
    deps: Deps,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
) -> StdResult<Vec<(Addr, User)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);
    let user_bucket: ReadonlyBucket<User> = bucket_read(deps.storage, PREFIX_USER);

    user_bucket
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|elem: StdResult<(Vec<u8>, User)>| {
            let (k, v) = elem.unwrap();
            let user = deps.api.addr_humanize(&CanonicalAddr::from(k))?;
            Ok((user, v))
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<CanonicalAddr>) -> Option<Vec<u8>> {
    start_after.map(|address| {
        let mut v = address.as_slice().to_vec();
        v.push(1);
        v
    })
}
