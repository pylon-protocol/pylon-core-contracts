#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response, WasmMsg,
};
use pylon_token::gov_msg::{
    AirdropMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, PollMsg, QueryMsg, StakingMsg,
};

use crate::constant::POLL_EXECUTE_REPLY_ID;
use crate::error::ContractError;
use crate::state::poll::Poll;
use crate::{executions, queries};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    executions::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => executions::receive(deps, env, info, msg),
        ExecuteMsg::UpdateConfig {
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            proposal_deposit,
            snapshot_period,
        } => executions::update_config(
            deps,
            info,
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            proposal_deposit,
            snapshot_period,
        ),
        ExecuteMsg::Poll(msg) => match msg {
            PollMsg::CastVote {
                poll_id,
                vote,
                amount,
            } => executions::poll::cast_vote(deps, env, info, poll_id, vote.into(), amount),
            PollMsg::Execute { poll_id } => executions::poll::execute(deps, env, poll_id),
            PollMsg::ExecuteMsgs { poll_id } => {
                executions::poll::execute_messages(deps, env, info, poll_id)
            }
            PollMsg::Snapshot { poll_id } => executions::poll::snapshot(deps, env, info, poll_id),
            PollMsg::End { poll_id } => executions::poll::end(deps, env, poll_id),
        },
        ExecuteMsg::Staking(msg) => match msg {
            StakingMsg::Unstake { amount } => Ok(Response::new()
                // 1. Update reward
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::Airdrop(AirdropMsg::Update {
                        target: Some(info.sender.to_string()),
                    }))?,
                    funds: vec![],
                }))
                // 2. Execute Unstake
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::Staking(StakingMsg::UnstakeInternal {
                        sender: info.sender.to_string(),
                        amount,
                    }))?,
                    funds: vec![],
                }))),
            StakingMsg::StakeInternal { sender, amount } => {
                executions::staking::stake_voting_tokens(deps, env, info, sender, amount)
            }
            StakingMsg::UnstakeInternal { sender, amount } => {
                executions::staking::withdraw_voting_tokens(deps, env, info, sender, amount)
            }
        },
        ExecuteMsg::Airdrop(msg) => match msg {
            AirdropMsg::Instantiate {
                start,
                period,
                reward_token,
                reward_amount,
            } => executions::airdrop::instantiate(
                deps,
                env,
                info,
                start,
                period,
                reward_token,
                reward_amount,
            ),
            AirdropMsg::Allocate {
                airdrop_id,
                recipient,
                allocate_amount,
            } => executions::airdrop::allocate(
                deps,
                env,
                info,
                airdrop_id,
                recipient,
                allocate_amount,
            ),
            AirdropMsg::Deallocate {
                airdrop_id,
                recipient,
                deallocate_amount,
            } => executions::airdrop::deallocate(
                deps,
                env,
                info,
                airdrop_id,
                recipient,
                deallocate_amount,
            ),
            AirdropMsg::Update { target } => executions::airdrop::update(deps, env, info, target),
            AirdropMsg::Claim { target } => executions::airdrop::claim(deps, env, info, target),
            AirdropMsg::ClaimInternal { sender, airdrop_id } => {
                executions::airdrop::claim_internal(deps, env, info, sender, airdrop_id)
            }
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        POLL_EXECUTE_REPLY_ID => {
            let poll_id: u64 = Poll::load_temp_id(deps.storage)?;
            executions::poll::fail(deps, poll_id)
        }
        _ => Err(ContractError::InvalidReplyId {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::ApiVersion {} => queries::query_api_version(deps),
        QueryMsg::Config {} => queries::config::query_config(deps),
        QueryMsg::State {} => queries::state::query_state(deps),
        QueryMsg::Staker { address } => queries::bank::query_staker(deps, env, address),
        QueryMsg::Stakers {
            start_after,
            limit,
            order,
        } => queries::bank::query_stakers(deps, env, start_after, limit, order),
        QueryMsg::Airdrop { airdrop_id } => queries::airdrop::query_airdrop(deps, airdrop_id),
        QueryMsg::Airdrops {
            start_after,
            limit,
            order_by,
        } => queries::airdrop::query_airdrops(deps, start_after, limit, order_by),
        QueryMsg::Poll { poll_id } => queries::poll::query_poll(deps, poll_id),
        QueryMsg::Polls {
            start_after,
            limit,
            order_by,
        } => queries::poll::query_polls(deps, start_after, limit, order_by),
        QueryMsg::PollsWithCategoryFilter {
            category_filter,
            start_after,
            limit,
            order_by,
        } => queries::poll::query_polls_with_category_filter(
            deps,
            category_filter.map(|x| x.into()),
            start_after,
            limit,
            order_by,
        ),
        QueryMsg::PollsWithStatusFilter {
            status_filter,
            start_after,
            limit,
            order_by,
        } => queries::poll::query_polls_with_status_filter(
            deps,
            status_filter.map(|x| x.into()),
            start_after,
            limit,
            order_by,
        ),
        QueryMsg::Voters {
            poll_id,
            start_after,
            limit,
            order_by,
        } => queries::poll::query_voters(deps, poll_id, start_after, limit, order_by),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    executions::migrate(deps, env, msg)
}
