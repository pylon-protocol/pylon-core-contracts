use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult,
    Querier, StdResult, Storage,
};
use pylon_launchpad::lockup_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};
use std::ops::Add;

use crate::handler::core as Core;
use crate::handler::query as Query;
use crate::handler::router as Router;
use crate::migrate::migration;
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
            temp_withdraw_start_time: 0,
            temp_withdraw_finish_time: 0,
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
        // common
        HandleMsg::Update { target } => Core::update(deps, env, target),
        // router
        HandleMsg::Receive(msg) => Router::receive(deps, env, msg),
        HandleMsg::Withdraw { amount } => Router::withdraw(deps, env, amount),
        HandleMsg::Claim {} => Router::claim(deps, env),
        // internal
        HandleMsg::DepositInternal { sender, amount } => {
            Core::deposit_internal(deps, env, sender, amount)
        }
        HandleMsg::WithdrawInternal { sender, amount } => {
            Core::withdraw_internal(deps, env, sender, amount)
        }
        HandleMsg::ClaimInternal { sender } => Core::claim_internal(deps, env, sender),
        // owner
        HandleMsg::Configure {
            owner,
            start_time,
            cliff_time,
            finish_time,
        } => Core::configure(deps, env, owner, start_time, cliff_time, finish_time),
        HandleMsg::AddReward { amount } => Core::add_reward(deps, env, amount),
        HandleMsg::SubReward { amount } => Core::sub_reward(deps, env, amount),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Query::config(deps),
        QueryMsg::Reward {} => Query::reward(deps),
        QueryMsg::BalanceOf { owner } => Query::balance_of(deps, owner),
        QueryMsg::ClaimableReward { owner, timestamp } => {
            Query::claimable_reward(deps, owner, timestamp)
        }
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _: Env,
    msg: MigrateMsg,
) -> MigrateResult {
    // migration(deps, msg)
    Ok(MigrateResponse::default())
}
