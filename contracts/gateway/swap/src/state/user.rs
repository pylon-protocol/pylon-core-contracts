use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{CanonicalAddr, Deps, Order, StdResult, Storage};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use pylon_utils::common::OrderBy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static PREFIX_USER: &[u8] = b"user";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub whitelisted: bool,
    pub swapped_in: Uint256,
    pub swapped_out: Uint256,
    pub swapped_out_claimed: Uint256,
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
            whitelisted: false,
            swapped_in: Uint256::zero(),
            swapped_out: Uint256::zero(),
            swapped_out_claimed: Uint256::zero(),
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
    order_by: Option<OrderBy>,
) -> StdResult<Vec<(String, User)>> {
    let start = start_after.map(|x| {
        let mut v = x.as_slice().to_vec();
        v.push(1);
        v
    });
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let order = order_by.map(Order::from).unwrap_or(Order::Ascending);
    let user_bucket: ReadonlyBucket<User> = bucket_read(deps.storage, PREFIX_USER);

    user_bucket
        .range(start.as_deref(), None, order)
        .take(limit)
        .map(|elem: StdResult<(Vec<u8>, User)>| {
            let (k, v) = elem.unwrap();
            let user = deps
                .api
                .addr_humanize(&CanonicalAddr::from(k))
                .unwrap()
                .to_string();
            Ok((user, v))
        })
        .collect()
}
