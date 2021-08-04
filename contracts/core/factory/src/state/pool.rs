use cosmwasm_std::{CanonicalAddr, Order, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static PREFIX_POOL: &[u8] = b"pool";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Neutral,
    Ready,
    Deployed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Pool {
    pub id: u64,
    pub status: Status,
    pub address: CanonicalAddr,
}

pub fn store<S: Storage>(storage: &mut S, id: u64, pool: &Pool) -> StdResult<()> {
    let key = &id.to_be_bytes()[..];
    Bucket::new(PREFIX_POOL, storage).save(key, pool)
}

pub fn read<S: ReadonlyStorage>(storage: &S, id: u64) -> StdResult<Pool> {
    let key = &id.to_be_bytes()[..];
    match ReadonlyBucket::new(PREFIX_POOL, storage).may_load(key)? {
        Some(pool) => Ok(pool),
        None => Ok(Pool {
            id: 0,
            status: Status::Neutral,
            address: CanonicalAddr::default(),
        }),
    }
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn batch_read<S: ReadonlyStorage>(
    storage: &S,
    start: u64,
    limit: Option<u32>,
) -> StdResult<Vec<Pool>> {
    let key = &start.to_be_bytes()[..];
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    ReadonlyBucket::new(PREFIX_POOL, storage)
        .range(Option::from(key), None, Order::Ascending)
        .take(limit)
        .map(|elem: StdResult<(_, Pool)>| {
            let (_, v) = elem?;
            Ok(Pool {
                id: v.id,
                status: v.status,
                address: v.address,
            })
        })
        .collect()
}

pub fn remove<S: Storage>(storage: &mut S, id: u64) {
    let key = &id.to_be_bytes()[..];
    Bucket::<S, Pool>::new(PREFIX_POOL, storage).remove(key)
}
