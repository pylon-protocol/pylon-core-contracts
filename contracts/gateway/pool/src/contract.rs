use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult,
    Querier, StdError, StdResult, Storage,
};
use pylon_gateway::pool_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};
use std::ops::{Add, Div};

use crate::handler::core as Core;
use crate::handler::query as Query;
use crate::handler::router as Router;
use crate::state::{config, state};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if msg.sale_period <= 0 {
        return Err(StdError::generic_err(
            "GatewayPool: sale period cannot be zero",
        ));
    }
    if msg.cliff_period.add(msg.vesting_period) != msg.sale_period {
        return Err(StdError::generic_err(
            "GatewayPool: sale period must equals with cliff + vesting period",
        ));
    }
    if msg.sale_amount.is_zero() {
        return Err(StdError::generic_err(
            "GatewayPool: sale amount canont be zero",
        ));
    }
    if msg.unbonding_period.ne(&0) {
        return Err(StdError::generic_err(
            "GatewayPool: unbonding period feature is not implemented",
        ));
    }

    config::store(
        &mut deps.storage,
        &config::Config {
            owner: deps.api.canonical_address(&env.message.sender)?,
            start_time: msg.start_time.clone(),
            finish_time: msg.start_time.add(msg.sale_period),

            depositable: msg.depositable.clone(),
            withdrawable: msg.withdrawable.clone(),
            cliff_period: msg.cliff_period.clone(),
            vesting_period: msg.vesting_period.clone(),
            unbonding_period: msg.unbonding_period.clone(),
            reward_rate: Decimal256::from_uint256(
                msg.sale_amount
                    .div(Decimal256::from_uint256(Uint256::from(msg.sale_period))),
            ),

            staking_token: deps.api.canonical_address(&msg.staking_token)?,
            reward_token: deps.api.canonical_address(&msg.reward_token)?,
        },
    )?;

    state::store(
        &mut deps.storage,
        &state::State {
            total_deposit: Uint256::zero(),
            last_update_time: msg.start_time,
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
        // router
        HandleMsg::Receive(msg) => Router::receive(deps, env, msg),
        HandleMsg::Withdraw { amount } => Router::withdraw(deps, env, amount),
        HandleMsg::ClaimReward {} => Router::claim_reward(deps, env),
        HandleMsg::ClaimWithdrawal {} => Router::claim_withdrawal(deps, env),
        // internal
        HandleMsg::Update { target } => Core::update(deps, env, target),
        HandleMsg::DepositInternal { sender, amount } => {
            Core::deposit_internal(deps, env, sender, amount)
        }
        HandleMsg::WithdrawInternal { sender, amount } => {
            Core::withdraw_internal(deps, env, sender, amount)
        }
        HandleMsg::ClaimRewardInternal { sender } => Core::claim_reward_internal(deps, env, sender),
        HandleMsg::ClaimWithdrawalInternal { sender } => {
            Core::claim_withdrawal_internal(deps, env, sender)
        }
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Query::config(deps),
        QueryMsg::Reward {} => Query::reward(deps),
        QueryMsg::BalanceOf { address } => Query::balance_of(deps, address),
        QueryMsg::ClaimableReward { address, timestamp } => {
            Query::claimable_reward(deps, address, timestamp)
        }
        QueryMsg::ClaimableWithdrawal { address, timestamp } => {
            Query::claimable_withdrawal(deps, address, timestamp)
        }
        QueryMsg::PendingWithdrawals {
            address,
            page,
            limit,
        } => Query::pending_withdrawals(deps, address, page, limit),
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _: &mut Extern<S, A, Q>,
    _: Env,
    _: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
