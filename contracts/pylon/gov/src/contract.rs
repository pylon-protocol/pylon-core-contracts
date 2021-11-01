#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, Uint128,
};
use cw20::Cw20ReceiveMsg;
use pylon_token::gov::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::constant::POLL_EXECUTE_REPLY_ID;
use crate::error::ContractError;
use crate::handler::config::{query_config, update_config};
use crate::handler::poll::{
    create_poll, fail_poll, handle_poll_msg, query_poll, query_polls, query_voters,
};
use crate::handler::staking::{
    handle_staking_msg, query_staker, query_stakers, stake_voting_tokens,
};
use crate::handler::state::query_state;
use crate::querier::poll::read_tmp_poll_id;
use crate::state::config::{config_r, config_w, Config};
use crate::state::state::{state_w, State};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        pylon_token: deps.api.addr_canonicalize(msg.voting_token.as_str())?,
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        timelock_period: msg.timelock_period,
        expiration_period: 0u64, // Deprecated
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
    };
    config.validate()?;

    let state = State {
        contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        poll_count: 0,
        total_share: Uint128::zero(),
        total_deposit: Uint128::zero(),
    };

    config_w(deps.storage).save(&config)?;
    state_w(deps.storage).save(&state)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::UpdateConfig {
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            proposal_deposit,
            snapshot_period,
        } => update_config(
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
        ExecuteMsg::Poll(msg) => handle_poll_msg(deps, env, info, msg),
        ExecuteMsg::Staking(msg) => handle_staking_msg(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        POLL_EXECUTE_REPLY_ID => {
            let poll_id: u64 = read_tmp_poll_id(deps.storage)?;
            fail_poll(deps, poll_id)
        }
        _ => Err(ContractError::InvalidReplyId {}),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // only asset contract can execute this message
    let config = config_r(deps.storage).load()?;
    if config.pylon_token != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::StakeVotingTokens {}) => {
            let api = deps.api;
            stake_voting_tokens(deps, api.addr_validate(&cw20_msg.sender)?, cw20_msg.amount)
        }
        Ok(Cw20HookMsg::CreatePoll {
            title,
            description,
            link,
            execute_msgs,
        }) => create_poll(
            deps,
            env,
            cw20_msg.sender,
            cw20_msg.amount,
            title,
            description,
            link,
            execute_msgs,
        ),
        _ => Err(ContractError::DataShouldBeGiven {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => query_config(deps),
        QueryMsg::State {} => query_state(deps),
        QueryMsg::Staker { address } => query_staker(deps, address),
        QueryMsg::Stakers {
            start_after,
            limit,
            order,
        } => query_stakers(deps, start_after, limit, order),
        QueryMsg::Poll { poll_id } => query_poll(deps, poll_id),
        QueryMsg::Polls {
            filter,
            start_after,
            limit,
            order_by,
        } => query_polls(deps, filter, start_after, limit, order_by),
        QueryMsg::Voters {
            poll_id,
            start_after,
            limit,
            order_by,
        } => query_voters(deps, poll_id, start_after, limit, order_by),
    }
}
