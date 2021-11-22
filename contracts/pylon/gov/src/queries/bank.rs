use cosmwasm_std::{to_binary, Deps, Env, Uint128};
use pylon_token::common::OrderBy;
use pylon_token::gov_msg::{ClaimableAirdrop, VoterInfo as GovVoterInfo};
use pylon_token::gov_resp::{StakerResponse, StakersResponse};
use terraswap::querier::query_token_balance;

use crate::executions::airdrop::{calculate_reward_per_token, calculate_rewards};
use crate::queries::QueryResult;
use crate::state::airdrop::{Airdrop, Reward as AirdropReward};
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
        env.contract.address.clone(),
    )?
    .checked_sub(state.total_deposit)?;

    Ok(to_binary(&to_response(
        &deps,
        &env,
        address.as_str(),
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
        env.contract.address.clone(),
    )?
    .checked_sub(state.total_deposit)?;

    let stakers: Vec<(String, StakerResponse)> = managers
        .iter()
        .map(|(address, token_manager)| -> (String, StakerResponse) {
            let address = deps.api.addr_humanize(address).unwrap();
            (
                address.to_string(),
                to_response(
                    &deps,
                    &env,
                    address.as_str(),
                    &state.total_share,
                    &total_balance,
                    token_manager,
                ),
            )
        })
        .collect();

    Ok(to_binary(&StakersResponse { stakers })?)
}

fn to_response(
    deps: &Deps,
    env: &Env,
    staker: &str,
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

    let claimable_airdrop = AirdropReward::load_range(
        deps.storage,
        &deps.api.addr_validate(staker).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();

    let claimable_airdrop = claimable_airdrop
        .iter()
        .map(|(airdrop_id, airdrop_reward)| {
            let mut airdrop = Airdrop::load(deps.storage, airdrop_id).unwrap();
            let applicable_time = airdrop.applicable_time(&env.block);

            airdrop.state.reward_per_token_stored =
                if airdrop.finish() == airdrop.state.last_update_time {
                    airdrop.state.reward_per_token_stored // because it's already latest
                } else {
                    airdrop.state.reward_per_token_stored
                        + calculate_reward_per_token(
                            &applicable_time,
                            total_share,
                            &airdrop.config.reward_rate,
                            &airdrop.state.last_update_time,
                        )
                        .unwrap()
                };
            airdrop.state.last_update_time = applicable_time;

            let mut airdrop_reward = airdrop_reward.clone();
            airdrop_reward.reward = calculate_rewards(
                &applicable_time,
                total_share,
                &token_manager.share,
                &airdrop,
                &airdrop_reward,
            )
            .unwrap();
            airdrop_reward.reward_per_token_paid = airdrop.state.reward_per_token_stored;

            (
                *airdrop_id,
                ClaimableAirdrop {
                    token: airdrop.config.reward_token.to_string(),
                    amount: airdrop_reward.reward,
                },
            )
        })
        .filter(|(_, airdrop_reward)| !airdrop_reward.amount.is_zero())
        .collect();

    StakerResponse {
        balance,
        share: token_manager.share,
        locked_balance,
        claimable_airdrop,
    }
}
