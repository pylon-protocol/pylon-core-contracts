use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Order, Querier, ReadonlyStorage, StdResult, Storage,
    Uint128,
};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use pylon_token::gov::VoterInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static PREFIX_BANK: &[u8] = b"bank";

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenManager {
    pub share: Uint128,                        // total staked balance
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}

pub fn store<S: Storage>(storage: &mut S) -> Bucket<S, TokenManager> {
    bucket(PREFIX_BANK, storage)
}

pub fn read<S: ReadonlyStorage>(storage: &S) -> ReadonlyBucket<S, TokenManager> {
    bucket_read(PREFIX_BANK, storage)
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn batch_read<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
    order: Option<Order>,
) -> StdResult<Vec<(HumanAddr, TokenManager)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start(start_after);
    let order = order.unwrap_or(Order::Ascending);

    ReadonlyBucket::new(PREFIX_BANK, &deps.storage)
        .range(start.as_deref(), None, order)
        .take(limit)
        .map(|elem: StdResult<(Vec<u8>, TokenManager)>| {
            let (k, v) = elem.unwrap();
            let staker = deps.api.human_address(&CanonicalAddr::from(k))?;
            Ok((staker, v))
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
