use cosmwasm_std::{Binary, CanonicalAddr, ReadonlyStorage, StdResult, Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use pylon_token::common::OrderBy;
use pylon_token::gov::{PollStatus, VoterInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::state::{calc_range_end, calc_range_end_addr, calc_range_start, calc_range_start_addr};

static PREFIX_POLL_INDEXER: &[u8] = b"poll_indexer";
static PREFIX_POLL_VOTER: &[u8] = b"poll_voter";
static PREFIX_POLL: &[u8] = b"poll";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub id: u64,
    pub creator: CanonicalAddr,
    pub status: PollStatus,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub end_height: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub execute_data: Option<Vec<ExecuteData>>,
    pub deposit_amount: Uint128,
    /// Total balance at the end poll
    pub total_balance_at_end_poll: Option<Uint128>,
    pub staked_amount: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct ExecuteData {
    pub order: u64,
    pub contract: CanonicalAddr,
    pub msg: Binary,
}
impl Eq for ExecuteData {}

impl Ord for ExecuteData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialOrd for ExecuteData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ExecuteData {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

pub fn store<S: Storage>(storage: &mut S) -> Bucket<S, Poll> {
    bucket(PREFIX_POLL, storage)
}

pub fn read<S: ReadonlyStorage>(storage: &S) -> ReadonlyBucket<S, Poll> {
    bucket_read(PREFIX_POLL, storage)
}

pub fn store_indexer<'a, S: Storage>(
    storage: &'a mut S,
    status: &PollStatus,
) -> Bucket<'a, S, bool> {
    Bucket::multilevel(
        &[PREFIX_POLL_INDEXER, status.to_string().as_bytes()],
        storage,
    )
}

pub fn store_voter<S: Storage>(storage: &mut S, poll_id: u64) -> Bucket<S, VoterInfo> {
    Bucket::multilevel(&[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()], storage)
}

pub fn read_voter<S: ReadonlyStorage>(storage: &S, poll_id: u64) -> ReadonlyBucket<S, VoterInfo> {
    ReadonlyBucket::multilevel(&[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()], storage)
}

pub fn read_poll_voters<'a, S: ReadonlyStorage>(
    storage: &'a S,
    poll_id: u64,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<(CanonicalAddr, VoterInfo)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
        _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
    };

    let voters: ReadonlyBucket<'a, S, VoterInfo> =
        ReadonlyBucket::multilevel(&[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()], storage);
    voters
        .range(start.as_deref(), end.as_deref(), order_by.into())
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            Ok((CanonicalAddr::from(k), v))
        })
        .collect()
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_polls<'a, S: ReadonlyStorage>(
    storage: &'a S,
    filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<Poll>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
        _ => (None, calc_range_end(start_after), OrderBy::Desc),
    };

    if let Some(status) = filter {
        let poll_indexer: ReadonlyBucket<'a, S, bool> = ReadonlyBucket::multilevel(
            &[PREFIX_POLL_INDEXER, status.to_string().as_bytes()],
            storage,
        );
        poll_indexer
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| {
                let (k, _) = item?;
                read(storage).load(&k)
            })
            .collect()
    } else {
        let polls: ReadonlyBucket<'a, S, Poll> = ReadonlyBucket::new(PREFIX_POLL, storage);

        polls
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| {
                let (_, v) = item?;
                Ok(v)
            })
            .collect()
    }
}
