#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use pylon_gateway::pool_msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use pylon_gateway::time_range::TimeRange;
use std::ops::Add;

use crate::error::ContractError;
use crate::handler::configure as Config;
use crate::handler::core as Core;
use crate::handler::query as Query;
use crate::handler::router as Router;
use crate::state::{config, reward};

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    config::store(
        deps.storage,
        &config::Config {
            owner: info.sender.to_string(),
            // share
            share_token: msg.share_token,
            deposit_config: config::DepositConfig {
                time: TimeRange {
                    start: msg.start,
                    finish: 0,
                    inverse: false,
                },
                user_cap: Uint256::zero(),
                total_cap: Uint256::zero(),
            },
            withdraw_time: vec![TimeRange {
                start: msg.start,
                finish: msg.start.add(msg.period),
                inverse: true,
            }],
            // reward
            reward_token: msg.reward_token,
            claim_time: TimeRange {
                start: msg.start.add(msg.cliff),
                finish: 0,
                inverse: false,
            },
            distribution_config: config::DistributionConfig {
                time: TimeRange {
                    start: msg.start,
                    finish: msg.start.add(msg.period),
                    inverse: false,
                },
                reward_rate: Decimal256::from_ratio(msg.reward_amount, Uint256::from(msg.period)),
            },
            cap_strategy: msg.cap_strategy,
        },
    )?;

    reward::store(
        deps.storage,
        &reward::Reward {
            total_deposit: Uint256::zero(),
            last_update_time: msg.start,
            reward_per_token_stored: Decimal256::zero(),
        },
    )?;

    Ok(Response::default())
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // common
        ExecuteMsg::Update { target } => Core::update(deps, env, info, target),
        // router
        ExecuteMsg::Receive(msg) => Router::receive(deps, env, info, msg),
        ExecuteMsg::Withdraw { amount } => Router::withdraw(deps, env, info, amount),
        ExecuteMsg::Claim {} => Router::claim(deps, env, info),
        // internal
        ExecuteMsg::DepositInternal { sender, amount } => {
            Core::deposit_internal(deps, env, info, sender, amount)
        }
        ExecuteMsg::WithdrawInternal { sender, amount } => {
            Core::withdraw_internal(deps, env, info, sender, amount)
        }
        ExecuteMsg::ClaimInternal { sender } => Core::claim_internal(deps, env, info, sender),
        // owner
        ExecuteMsg::Configure(msg) => Config::configure(deps, env, info, msg),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Query::config(deps, env),
        QueryMsg::Stakers { start_after, limit } => Query::stakers(deps, env, start_after, limit),
        QueryMsg::Reward {} => Query::reward(deps, env),
        QueryMsg::BalanceOf { owner } => Query::balance_of(deps, env, owner),
        QueryMsg::ClaimableReward { owner } => Query::claimable_reward(deps, env, owner),
        QueryMsg::AvailableCapOf { address } => Query::available_cap_of(deps, env, address),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
