use cosmwasm_std::*;
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
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
    pub address: String,
}

pub fn store(storage: &mut dyn Storage, id: u64, pool: &Pool) -> StdResult<()> {
    let key = &id.to_be_bytes()[..];
    let mut pool_bucket: Bucket<Pool> = bucket(storage, PREFIX_POOL);

    pool_bucket.save(key, pool)
}

pub fn remove(storage: &mut dyn Storage, id: u64) {
    let key = &id.to_be_bytes()[..];
    let mut pool_bucket: Bucket<Pool> = bucket(storage, PREFIX_POOL);

    pool_bucket.remove(key)
}

pub fn read(storage: &dyn Storage, id: u64) -> StdResult<Pool> {
    let key = &id.to_be_bytes()[..];
    let pool_bucket: ReadonlyBucket<Pool> = bucket_read(storage, PREFIX_POOL);

    match pool_bucket.may_load(key)? {
        Some(pool) => Ok(pool),
        None => Ok(Pool {
            id: 0,
            status: Status::Neutral,
            address: "".to_string(),
        }),
    }
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn batch_read(
    storage: &dyn Storage,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Vec<Pool>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);
    let pool_bucket: ReadonlyBucket<Pool> = bucket_read(storage, PREFIX_POOL);

    pool_bucket
        .range(start.as_deref(), None, Order::Ascending)
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

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|x| {
        let mut v = x.to_be_bytes().to_vec();
        v.push(1);
        v
    })
}
