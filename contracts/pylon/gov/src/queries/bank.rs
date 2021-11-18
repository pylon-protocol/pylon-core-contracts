use cosmwasm_std::{to_binary, Deps, Env, Uint128};
use pylon_token::common::OrderBy;
use pylon_token::gov_msg::VoterInfo as GovVoterInfo;
use pylon_token::gov_resp::{StakerResponse, StakersResponse};
use terraswap::querier::query_token_balance;

use crate::queries::QueryResult;
use crate::state::bank::TokenManager;
use crate::state::config::Config;
use crate::state::poll::{Poll, PollStatus};
use crate::state::state::State;

pub fn query_staker(deps: Deps, env: Env, address: String) -> QueryResult {
    let config = Config::load(deps.storage)?;
    let state = State::load(deps.storage)?;
    let token_manager = TokenManager::load(deps.storage, &deps.api.addr_canonicalize(&address)?)?;

    let total_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.pylon_token)?,
        env.contract.address,
    )?
    .checked_sub(state.total_deposit)?;

    Ok(to_binary(&to_response(
        deps,
        &state.total_share,
        &total_balance,
        &token_manager,
    ))?)
}

pub fn query_stakers(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
    order: Option<OrderBy>,
) -> QueryResult {
    let state = State::load(deps.storage).unwrap();
    let config = Config::load(deps.storage).unwrap();
    let managers = TokenManager::load_range(
        deps.storage,
        start_after.map(|x| deps.api.addr_canonicalize(x.as_str()).unwrap()),
        limit,
        order,
    )?;

    let total_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.pylon_token)?,
        env.contract.address,
    )?
    .checked_sub(state.total_deposit)?;

    let stakers: Vec<(String, StakerResponse)> = managers
        .iter()
        .map(|(address, token_manager)| -> (String, StakerResponse) {
            (
                deps.api.addr_humanize(address).unwrap().to_string(),
                to_response(deps, &state.total_share, &total_balance, token_manager),
            )
        })
        .collect();

    Ok(to_binary(&StakersResponse { stakers })?)
}

fn to_response(
    deps: Deps,
    total_share: &Uint128,
    total_balance: &Uint128,
    token_manager: &TokenManager,
) -> StakerResponse {
    let balance = if !total_share.is_zero() {
        token_manager
            .share
            .multiply_ratio(*total_balance, *total_share)
    } else {
        Uint128::zero()
    };

    let locked_balance = token_manager
        .locked_balance
        .iter()
        .filter(|(poll_id, _)| {
            let poll = Poll::load(deps.storage, poll_id).unwrap();

            poll.status == PollStatus::InProgress
        })
        .map(|(poll_id, voter_info)| -> (u64, GovVoterInfo) {
            (
                *poll_id,
                GovVoterInfo {
                    vote: voter_info.vote.clone().into(),
                    balance: voter_info.balance,
                },
            )
        })
        .collect();

    StakerResponse {
        balance,
        share: token_manager.share,
        locked_balance,
        claimable_airdrop: vec![],
    }
}
