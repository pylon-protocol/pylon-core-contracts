use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult,
    Querier, StdResult, Storage,
};
use std::ops::Add;

use pylon_launchpad::lockup_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};

use crate::handler::core as CoreHandler;
use crate::handler::query as QueryHandler;
use crate::state;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    state::store_config(
        &mut deps.storage,
        &state::Config {
            owner: deps.api.canonical_address(&env.message.sender)?,
            share_token: deps.api.canonical_address(&msg.share_token)?,
            reward_token: deps.api.canonical_address(&msg.reward_token)?,
            start_time: msg.start,
            cliff_time: msg.start.add(msg.cliff),
            finish_time: msg.start.add(msg.period),
            reward_rate: msg.reward_rate,
        },
    )?;

    state::store_reward(
        &mut deps.storage,
        &state::Reward {
            total_deposit: Uint256::zero(),
            last_update_time: msg.start,
            reward_per_token_stored: Decimal256::zero(),
        },
    )?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        // core
        HandleMsg::Receive(msg) => CoreHandler::receive(deps, env, msg),
        HandleMsg::Update {} => CoreHandler::update(deps, &env, None),
        HandleMsg::Withdraw { amount } => CoreHandler::withdraw(deps, env, amount),
        HandleMsg::Claim {} => CoreHandler::claim(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::Reward {} => QueryHandler::reward(deps),
        QueryMsg::BalanceOf { owner } => QueryHandler::balance_of(deps, owner),
        QueryMsg::ClaimableReward { owner, timestamp } => {
            QueryHandler::claimable_reward(deps, owner, timestamp)
        }
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _: &mut Extern<S, A, Q>,
    _: Env,
    _: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
