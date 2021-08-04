use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Decimal, Env, Extern, HandleResponse, InitResponse,
    InitResult, Querier, StdError, StdResult, Storage, Uint128,
};
use pylon_token::gov::{HandleMsg, InitMsg, QueryMsg};

use crate::handler::{
    core as CoreHandler, poll as PollHandler, query as QueryHandler, staker as StakerHandler,
};
use crate::state::{config, state};

/// validate_quorum returns an error if the quorum is invalid
/// (we require 0-1)
fn validate_quorum(quorum: Decimal) -> StdResult<()> {
    if quorum > Decimal::one() {
        Err(StdError::generic_err("quorum must be 0 to 1"))
    } else {
        Ok(())
    }
}

/// validate_threshold returns an error if the threshold is invalid
/// (we require 0-1)
fn validate_threshold(threshold: Decimal) -> StdResult<()> {
    if threshold > Decimal::one() {
        Err(StdError::generic_err("threshold must be 0 to 1"))
    } else {
        Ok(())
    }
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> InitResult {
    validate_quorum(msg.quorum)?;
    validate_threshold(msg.threshold)?;

    let config = config::Config {
        pylon_token: CanonicalAddr::default(),
        owner: deps.api.canonical_address(&env.message.sender)?,
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        timelock_period: msg.timelock_period,
        expiration_period: msg.expiration_period,
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
    };

    let state = state::State {
        contract_addr: deps.api.canonical_address(&env.contract.address)?,
        poll_count: 0,
        total_share: Uint128::zero(),
        total_deposit: Uint128::zero(),
    };

    config::store(&mut deps.storage).save(&config)?;
    state::store(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Receive(msg) => CoreHandler::receive_cw20(deps, env, msg),
        HandleMsg::RegisterContracts { pylon_token } => {
            CoreHandler::register_contracts(deps, pylon_token)
        }
        HandleMsg::UpdateConfig {
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        } => CoreHandler::update_config(
            deps,
            env,
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        ),
        HandleMsg::WithdrawVotingTokens { amount } => {
            StakerHandler::withdraw_voting_tokens(deps, env, amount)
        }
        HandleMsg::CastVote {
            poll_id,
            vote,
            amount,
        } => PollHandler::cast_vote(deps, env, poll_id, vote, amount),
        HandleMsg::EndPoll { poll_id } => PollHandler::end(deps, env, poll_id),
        HandleMsg::ExecutePoll { poll_id } => PollHandler::execute(deps, env, poll_id),
        HandleMsg::ExpirePoll { poll_id } => PollHandler::expire(deps, env, poll_id),
        HandleMsg::SnapshotPoll { poll_id } => PollHandler::snapshot(deps, env, poll_id),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&QueryHandler::config(&deps)?),
        QueryMsg::State {} => to_binary(&QueryHandler::state(&deps)?),
        QueryMsg::Staker { address } => to_binary(&QueryHandler::staker(deps, address)?),
        QueryMsg::Poll { poll_id } => to_binary(&QueryHandler::poll(deps, poll_id)?),
        QueryMsg::Polls {
            filter,
            start_after,
            limit,
            order_by,
        } => to_binary(&QueryHandler::polls(
            deps,
            filter,
            start_after,
            limit,
            order_by,
        )?),
        QueryMsg::Voters {
            poll_id,
            start_after,
            limit,
            order_by,
        } => to_binary(&QueryHandler::voters(
            deps,
            poll_id,
            start_after,
            limit,
            order_by,
        )?),
    }
}
