use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult,
    Querier, StdResult, Storage,
};
use pylon_launchpad::lockup_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};

use crate::handler::configure as Config;
use crate::handler::core as Core;
use crate::handler::query as Query;
use crate::handler::router as Router;
use crate::state::{config, reward, time_range};
use std::ops::{Add, Mul};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    config::store(
        &mut deps.storage,
        &config::Config {
            owner: env.message.sender,
            // share
            share_token: msg.share_token,
            deposit_config: config::DepositConfig {
                time: time_range::TimeRange {
                    start: msg.start,
                    finish: msg.start.add(msg.period),
                    inverse: false,
                },
                user_cap: Uint256::zero(),
                total_cap: Uint256::zero(),
            },
            withdraw_time: time_range::TimeRange {
                start: msg.start,
                finish: msg.start.add(msg.period),
                inverse: true,
            },
            temp_deposit_config: config::DepositConfig::default(),
            temp_withdraw_time: time_range::TimeRange::default(),
            // reward
            reward_token: msg.reward_token,
            claim_time: time_range::TimeRange {
                start: msg.cliff,
                finish: msg.start.add(msg.period),
                inverse: false,
            },
            distribution_config: config::DistributionConfig {
                time: time_range::TimeRange {
                    start: msg.start,
                    finish: msg.start.add(msg.period),
                    inverse: false,
                },
                reward_rate: msg.reward_rate,
                total_reward_amount: Uint256::from(msg.period).mul(msg.reward_rate),
            },
        },
    )?;

    reward::store(
        &mut deps.storage,
        &reward::Reward {
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
        HandleMsg::Configure(msg) => Config::configure(deps, env, msg),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Query::config(deps),
        QueryMsg::Stakers {
            start_after,
            limit,
            timestamp,
        } => Query::stakers(
            deps,
            match start_after {
                Some(start_after) => {
                    Option::from(deps.api.canonical_address(&start_after).unwrap())
                }
                None => Option::None,
            },
            limit,
            timestamp,
        ),
        QueryMsg::Reward {} => Query::reward(deps),
        QueryMsg::BalanceOf { owner } => Query::balance_of(deps, owner),
        QueryMsg::ClaimableReward { owner, timestamp } => {
            Query::claimable_reward(deps, owner, timestamp)
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
