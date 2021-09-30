use cosmwasm_std::*;
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static PREFIX_ADAPTER: &[u8] = b"adapter";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Adapter {
    pub address: String,
}

pub fn store(storage: &mut dyn Storage, address: String, adapter: &Adapter) -> StdResult<()> {
    let mut adapter_bucket: Bucket<Adapter> = bucket(storage, PREFIX_ADAPTER);

    adapter_bucket.save(address.as_bytes(), adapter)
}

pub fn remove(storage: &mut dyn Storage, address: String) {
    let mut adapter_bucket: Bucket<Adapter> = bucket(storage, PREFIX_ADAPTER);

    adapter_bucket.remove(address.as_bytes())
}

pub fn read(storage: &dyn Storage, address: String) -> StdResult<Adapter> {
    let adapter_bucket: ReadonlyBucket<Adapter> = bucket_read(storage, PREFIX_ADAPTER);

    match adapter_bucket.may_load(address.as_bytes())? {
        Some(adapter) => Ok(adapter),
        None => Ok(Adapter {
            address: String::default(),
        }),
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn batch_read(
    storage: &dyn Storage,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Adapter>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);
    let adapter_bucket: ReadonlyBucket<Adapter> = bucket_read(storage, PREFIX_ADAPTER);

    adapter_bucket
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|elem: StdResult<(Vec<u8>, Adapter)>| {
            let (_, v) = elem.unwrap();
            Ok(v)
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<String>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_bytes().to_vec();
        v.push(1);
        v
    })
}
