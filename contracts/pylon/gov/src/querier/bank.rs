use cosmwasm_std::{CanonicalAddr, Deps, Order, StdResult, Uint128};
use pylon_token::common::OrderBy;
use pylon_token::gov_msg::PollStatus;
use pylon_token::gov_resp::{StakerResponse, StakersResponse};
use terraswap::querier::query_token_balance;

use crate::constant::{DEFAULT_QUERY_LIMIT, MAX_QUERY_LIMIT};
use crate::state::bank::{bank_r, TokenManager};
use crate::state::config::config_r;
use crate::state::poll::poll_r;
use crate::state::state::state_r;

pub fn staker(deps: Deps, address: String) -> StdResult<StakerResponse> {
    let addr_raw = deps.api.addr_canonicalize(&address).unwrap();
    let config = config_r(deps.storage).load()?;
    let state = state_r(deps.storage).load()?;
    let mut token_manager = bank_r(deps.storage)
        .may_load(addr_raw.as_slice())?
        .unwrap_or_default();

    // filter out not in-progress polls
    token_manager.locked_balance.retain(|(poll_id, _)| {
        let poll = poll_r(deps.storage).load(&poll_id.to_be_bytes()).unwrap();

        poll.status == PollStatus::InProgress
    });

    let total_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.pylon_token)?,
        deps.api.addr_humanize(&state.contract_addr)?,
    )?
    .checked_sub(state.total_deposit)?;

    Ok(StakerResponse {
        balance: if !state.total_share.is_zero() {
            token_manager
                .share
                .multiply_ratio(total_balance, state.total_share)
        } else {
            Uint128::zero()
        },
        share: token_manager.share,
        locked_balance: token_manager.locked_balance,
    })
}

pub fn stakers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    order: Option<OrderBy>,
) -> StdResult<StakersResponse> {
    let state = state_r(deps.storage).load().unwrap();
    let config = config_r(deps.storage).load().unwrap();
    let start = start_after
        .map(|x| deps.api.addr_canonicalize(x.as_str()).unwrap())
        .map(|x| {
            let mut v = x.as_slice().to_vec();
            v.push(1);
            v
        });
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let order = order.map(Order::from).unwrap_or(Order::Ascending);

    let total_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.pylon_token)?,
        deps.api.addr_humanize(&state.contract_addr)?,
    )?
    .checked_sub(state.total_deposit)?;

    let stakers: Vec<(String, StakerResponse)> = bank_r(deps.storage)
        .range(start.as_deref(), None, order)
        .take(limit)
        .map(|elem: StdResult<(Vec<u8>, TokenManager)>| {
            let (k, mut v) = elem.unwrap();
            // filter out not in-progress polls
            v.locked_balance.retain(|(poll_id, _)| {
                let poll = poll_r(deps.storage).load(&poll_id.to_be_bytes()).unwrap();

                poll.status == PollStatus::InProgress
            });

            (
                deps.api
                    .addr_humanize(&CanonicalAddr::from(k))
                    .unwrap()
                    .to_string(),
                StakerResponse {
                    balance: if !state.total_share.is_zero() {
                        v.share.multiply_ratio(total_balance, state.total_share)
                    } else {
                        Uint128::zero()
                    },
                    share: v.share,
                    locked_balance: v.locked_balance,
                },
            )
        })
        .collect();
    Ok(StakersResponse { stakers })
}
