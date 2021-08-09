use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    Api, CanonicalAddr, Extern, Order, Querier, ReadonlyStorage, StdResult, Storage,
};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Add;

pub static PREFIX_WITHDRAWAL: &[u8] = b"withdrawal";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Withdrawal {
    pub amount: Uint256,
    pub accumulated: Uint256,
    pub period: u64,
    pub emitted: u64,
}

impl Withdrawal {
    pub fn is_claimable(&self, blocktime: &u64) -> bool {
        self.emitted.add(self.period).gt(blocktime)
    }
}

fn to_key(owner: &CanonicalAddr, index: u64) -> Vec<u8> {
    [owner.as_slice(), ":".as_bytes(), &index.to_be_bytes()].concat()
}

pub fn store<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    index: u64,
    withdrawal: &Withdrawal,
) -> StdResult<()> {
    let key = &to_key(owner, index)[..];
    Bucket::new(PREFIX_WITHDRAWAL, storage).save(key, withdrawal)
}

pub fn read<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    index: u64,
) -> StdResult<Withdrawal> {
    let key = &to_key(owner, index)[..];
    match ReadonlyBucket::new(PREFIX_WITHDRAWAL, storage).may_load(key)? {
        Some(withdrawal) => Ok(withdrawal),
        None => Ok(Withdrawal {
            amount: Uint256::zero(),
            accumulated: Uint256::zero(),
            period: 0,
            emitted: 0,
        }),
    }
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn batch_read<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: &CanonicalAddr,
    start: u64,
    limit: Option<u32>,
) -> StdResult<Vec<Withdrawal>> {
    let key = &to_key(owner, start)[..];
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    ReadonlyBucket::new(PREFIX_WITHDRAWAL, &deps.storage)
        .range(Option::from(key), None, Order::Ascending)
        .take(limit)
        .map(|elem: StdResult<(_, Withdrawal)>| {
            let (_, v) = elem?;
            Ok(Withdrawal {
                amount: v.amount,
                accumulated: v.accumulated,
                period: v.period,
                emitted: v.emitted,
            })
        })
        .collect()
}
