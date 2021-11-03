use cosmwasm_std::{CanonicalAddr, StdResult, Storage};
use cosmwasm_storage::ReadonlyBucket;
use pylon_token::common::OrderBy;
use pylon_token::gov_msg::{PollStatus, VoterInfo};

use crate::constant::{DEFAULT_QUERY_LIMIT, MAX_QUERY_LIMIT};
use crate::state::poll::{poll_indexed_by_status_r, poll_r, poll_voter_r, Poll};

pub fn polls(
    storage: &dyn Storage,
    category_filter: Option<String>,
    status_filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<Poll>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
        _ => (None, calc_range_end(start_after), OrderBy::Desc),
    };

    if let Some(status) = status_filter {
        let poll_indexer_store = poll_indexed_by_status_r(storage, &status);
        let polls = poll_indexer_store
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| {
                let (k, _) = item?;
                poll_r(storage).load(&k)
            });

        if let Some(category) = category_filter {
            polls
                .filter(|item| item.as_ref().unwrap().category == category)
                .collect()
        } else {
            polls.collect()
        }
    } else {
        let poll_store = poll_r(storage);
        let polls = poll_store
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| -> StdResult<Poll> {
                let (_, v) = item?;
                Ok(v)
            });

        if let Some(category) = category_filter {
            polls
                .filter(|item| item.as_ref().unwrap().category == category)
                .collect()
        } else {
            polls.collect()
        }
    }
}

pub fn poll_voters<'a>(
    storage: &'a dyn Storage,
    poll_id: u64,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<(CanonicalAddr, VoterInfo)>> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
        _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
    };

    let poll_voters: ReadonlyBucket<'a, VoterInfo> = poll_voter_r(storage, poll_id);

    poll_voters
        .range(start.as_deref(), end.as_deref(), order_by.into())
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            Ok((CanonicalAddr::from(k), v))
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| {
        let mut v = id.to_be_bytes().to_vec();
        v.push(1);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end(start_after: Option<u64>) -> Option<Vec<u8>> {
    start_after.map(|id| id.to_be_bytes().to_vec())
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start_addr(start_after: Option<CanonicalAddr>) -> Option<Vec<u8>> {
    start_after.map(|address| {
        let mut v = address.as_slice().to_vec();
        v.push(1);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end_addr(start_after: Option<CanonicalAddr>) -> Option<Vec<u8>> {
    start_after.map(|address| address.as_slice().to_vec())
}
