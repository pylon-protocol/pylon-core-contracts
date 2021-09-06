use cosmwasm_std::{
    Api, CanonicalAddr, Extern, Order, Querier, ReadonlyStorage, StdResult, Storage,
};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static PREFIX_ADAPTER: &[u8] = b"adapter";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Adapter {
    pub address: CanonicalAddr,
}

pub fn store<S: Storage>(
    storage: &mut S,
    address: CanonicalAddr,
    adapter: &Adapter,
) -> StdResult<()> {
    Bucket::new(PREFIX_ADAPTER, storage).save(address.as_slice(), adapter)
}

pub fn remove<S: Storage>(storage: &mut S, address: CanonicalAddr) {
    Bucket::<S, Adapter>::new(PREFIX_ADAPTER, storage).remove(address.as_slice())
}

pub fn read<S: ReadonlyStorage>(storage: &S, address: CanonicalAddr) -> StdResult<Adapter> {
    match ReadonlyBucket::new(PREFIX_ADAPTER, storage).may_load(address.as_slice())? {
        Some(adapter) => Ok(adapter),
        None => Ok(Adapter {
            address: CanonicalAddr::default(),
        }),
    }
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn batch_read<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
) -> StdResult<Vec<Adapter>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);

    ReadonlyBucket::new(PREFIX_ADAPTER, &deps.storage)
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|elem: StdResult<(Vec<u8>, Adapter)>| {
            let (_, v) = elem.unwrap();
            Ok(v)
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
