use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult,
    Querier, StdResult, Storage, Uint128,
};
use std::ops::Add;

use crate::handler_exec as ExecHandler;
use crate::handler_query as QueryHandler;
use crate::msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};
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
            start_time: msg.start_time.clone(),
            finish_time: msg.start_time.add(msg.period),
            open_deposit: msg.open_deposit.clone(),
            open_withdraw: msg.open_withdraw.clone(),
            open_claim: msg.open_claim.clone(),
            reward_rate: msg.reward_rate.clone(),
        },
    )?;

    state::store_reward(
        &mut deps.storage,
        &state::Reward {
            total_deposit: Uint128::zero(),
            last_update_time: 0,
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
        HandleMsg::Receive(msg) => ExecHandler::receive(deps, &env, msg),
        HandleMsg::Update {} => ExecHandler::update(deps, &env, None),
        HandleMsg::Withdraw { amount } => {
            ExecHandler::withdraw(deps, &env, &env.message.sender, amount)
        }
        HandleMsg::Claim {} => ExecHandler::claim(deps, &env.message.sender, &env),
        HandleMsg::Exit {} => ExecHandler::exit(deps, &env.message.sender, &env),
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
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
