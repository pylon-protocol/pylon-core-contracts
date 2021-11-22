use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Attribute, DepsMut, Env, MessageInfo, Response};
use pylon_gateway::pool_msg::ConfigureMsg;
use pylon_gateway::time_range::TimeRange;
use std::ops::{Add, Div, Sub};

use crate::error::ContractError;
use crate::state::config;

const MAX_WITHDRAW_STRATEGY: usize = 4;

pub fn configure(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ConfigureMsg,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).unwrap();
    if config.owner.ne(&info.sender.to_string()) {
        return Err(ContractError::Unauthorized {
            action: "configure".to_string(),
            expected: config.owner,
            actual: info.sender.to_string(),
        });
    }

    match msg {
        ConfigureMsg::Pool {
            owner,
            share_token,
            reward_token,
        } => {
            let mut config = config::read(deps.storage).unwrap();

            if let Some(v) = owner {
                config.owner = v;
            }
            if let Some(v) = share_token {
                config.share_token = v;
            }
            if let Some(v) = reward_token {
                config.reward_token = v;
            }

            config::store(deps.storage, &config).unwrap();

            Ok(Response::default())
        }
        ConfigureMsg::Deposit {
            start,
            finish,
            user_cap,
            total_cap,
        } => {
            let mut config = config::read(deps.storage).unwrap();
            let mut deposit_config = config.deposit_config;

            let mut attrs = vec![Attribute::new("action", "configure_deposit")];
            attrs.append(&mut deposit_config.time.configure(start, finish));
            if let Some(user_cap) = user_cap {
                deposit_config.user_cap = user_cap;
                attrs.push(Attribute::new("new_user_cap", user_cap.to_string()));
            }
            if let Some(total_cap) = total_cap {
                deposit_config.total_cap = total_cap;
                attrs.push(Attribute::new("new_total_cap", total_cap.to_string()));
            }

            config.deposit_config = deposit_config;
            config::store(deps.storage, &config).unwrap();

            Ok(Response::new().add_attributes(attrs))
        }
        ConfigureMsg::Withdraw { strategy } => {
            let mut config = config::read(deps.storage).unwrap();

            if strategy.len().gt(&MAX_WITHDRAW_STRATEGY) {
                return Err(ContractError::WithdrawStrategyLengthExceeded {
                    limit: MAX_WITHDRAW_STRATEGY,
                    length: strategy.len(),
                });
            }

            config.withdraw_time = strategy
                .iter()
                .map(|(start, finish, inverse)| TimeRange {
                    start: *start,
                    finish: *finish,
                    inverse: *inverse,
                })
                .collect();
            config::store(deps.storage, &config).unwrap();

            Ok(Response::new().add_attribute("action", "configure_withdraw"))
        }
        ConfigureMsg::Claim { start, finish } => {
            let mut config = config::read(deps.storage).unwrap();
            let mut claim_time = config.claim_time;

            let mut attrs = vec![Attribute::new("action", "configure_claim")];
            attrs.append(&mut claim_time.configure(start, finish));

            config.claim_time = claim_time;
            config::store(deps.storage, &config).unwrap();

            Ok(Response::new().add_attributes(attrs))
        }
        ConfigureMsg::AddReward { amount } => adjust_reward(deps, env, amount, false),
        ConfigureMsg::SubReward { amount } => adjust_reward(deps, env, amount, true),
    }
}

fn adjust_reward(
    deps: DepsMut,
    env: Env,
    amount: Uint256,
    remove: bool,
) -> Result<Response, ContractError> {
    let action = if remove { "sub_reward" } else { "add_reward" };

    let mut config = config::read(deps.storage).unwrap();
    let mut dist_config = config.distribution_config;

    if remove && env.block.time.seconds().gt(&dist_config.time.finish) {
        return Err(ContractError::SaleFinished {
            now: env.block.time.seconds(),
            finished: dist_config.time.finish,
        });
    }

    let reward_rate_before = dist_config.reward_rate;
    let remaining = Uint256::from(
        dist_config
            .time
            .finish
            .sub(dist_config.applicable_start_time(env.block.time.seconds())),
    );
    dist_config.reward_rate = if remove {
        dist_config
            .reward_rate
            .sub(Decimal256::from_uint256(amount).div(Decimal256::from_uint256(remaining)))
    } else {
        dist_config
            .reward_rate
            .add(Decimal256::from_uint256(amount).div(Decimal256::from_uint256(remaining)))
    };
    config.distribution_config = dist_config;
    config::store(deps.storage, &config).unwrap();

    Ok(Response::new()
        .add_attribute("action", action.to_string())
        .add_attribute("reward_rate_before", reward_rate_before.to_string())
        .add_attribute(
            "reward_rate_after",
            config.distribution_config.reward_rate.to_string(),
        ))
}
