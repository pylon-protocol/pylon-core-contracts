use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    log, Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage,
};
use pylon_gateway::pool_msg::{ConfigureMsg, DistributionMsg};
use std::ops::{Add, Div, Mul, Sub};

use crate::handler::validate_sender;
use crate::state::config;
use crate::state::time_range::TimeRange;

const MAX_WITHDRAW_STRATEGY: usize = 4;

pub fn configure<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: ConfigureMsg,
) -> StdResult<HandleResponse> {
    match msg {
        ConfigureMsg::Owner { address } => configure_owner(deps, env, address),
        ConfigureMsg::Deposit {
            start,
            finish,
            user_cap,
            total_cap,
        } => configure_deposit(deps, env, start, finish, user_cap, total_cap),
        ConfigureMsg::Withdraw { strategy } => {
            let mut config = config::read(&deps.storage).unwrap();
            validate_sender(&env, &config.owner, "configure_withdraw")?;

            print!("{}, {}", strategy.len(), MAX_WITHDRAW_STRATEGY);
            if strategy.len().gt(&MAX_WITHDRAW_STRATEGY) {
                return Err(StdError::generic_err(format!(
                    "Lockup: withdraw strategy length exceeds limit. limit: {}, now: {}",
                    MAX_WITHDRAW_STRATEGY,
                    strategy.len(),
                )));
            }

            config.withdraw_time = strategy
                .iter()
                .map(|(start, finish, inverse)| TimeRange {
                    start: *start,
                    finish: *finish,
                    inverse: *inverse,
                })
                .collect();
            config::store(&mut deps.storage, &config).unwrap();

            Ok(HandleResponse {
                messages: vec![],
                log: vec![log("action", "configure_withdraw")],
                data: None,
            })
        }
        ConfigureMsg::Claim { start, finish } => {
            let mut config = config::read(&deps.storage).unwrap();
            let mut claim_time = config.claim_time;
            validate_sender(&env, &config.owner, "configure_claim")?;

            let mut logs = vec![log("action", "configure_claim")];
            logs.append(&mut claim_time.configure(start, finish));

            config.claim_time = claim_time;
            config::store(&mut deps.storage, &config).unwrap();

            Ok(HandleResponse {
                messages: vec![],
                log: logs,
                data: None,
            })
        }
        ConfigureMsg::Distribution(msg) => configure_distribution(deps, env, msg),
    }
}

fn configure_owner<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config = config::read(&deps.storage).unwrap();
    validate_sender(&env, &config.owner, "configure_owner")?;

    let prev_owner = config.owner;
    config.owner = owner;

    config::store(&mut deps.storage, &config).unwrap();

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "configure_owner"),
            log("prev_owner", prev_owner),
            log("next_owner", config.owner),
        ],
        data: None,
    })
}

fn configure_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    start: Option<u64>,
    finish: Option<u64>,
    user_cap: Option<Uint256>,
    total_cap: Option<Uint256>,
) -> StdResult<HandleResponse> {
    let mut config = config::read(&deps.storage).unwrap();
    let mut deposit_config = config.deposit_config;
    validate_sender(&env, &config.owner, "configure_deposit")?;

    let mut logs = vec![log("action", "configure_deposit")];
    logs.append(&mut deposit_config.time.configure(start, finish));
    if let Some(user_cap) = user_cap {
        deposit_config.user_cap = user_cap;
        logs.push(log("new_user_cap", user_cap));
    }
    if let Some(total_cap) = total_cap {
        deposit_config.total_cap = total_cap;
        logs.push(log("new_total_cap", total_cap));
    }

    config.deposit_config = deposit_config;
    config::store(&mut deps.storage, &config).unwrap();

    Ok(HandleResponse {
        messages: vec![],
        log: logs,
        data: None,
    })
}

fn configure_distribution<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: DistributionMsg,
) -> StdResult<HandleResponse> {
    match msg {
        DistributionMsg::AddReward { amount } => adjust_reward(deps, env, amount, false),
        DistributionMsg::SubReward { amount } => adjust_reward(deps, env, amount, true),
        DistributionMsg::LengthenPeriod { time } => adjust_period(deps, env, time, false),
        DistributionMsg::ShortenPeriod { time } => adjust_period(deps, env, time, true),
    }
}

fn adjust_reward<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint256,
    remove: bool,
) -> StdResult<HandleResponse> {
    let action = if remove { "sub_reward" } else { "add_reward" };

    let mut config = config::read(&deps.storage).unwrap();
    let mut dist_config = config.distribution_config;
    validate_sender(&env, &config.owner, action)?;

    if remove && env.block.time.gt(&dist_config.time.finish) {
        return Err(StdError::generic_err(format!(
            "Lockup: sale finished. execution_time: {}, finish_time: {}",
            env.block.time, dist_config.time.finish,
        )));
    }

    let reward_rate_before = dist_config.reward_rate;
    let remaining = Uint256::from(
        dist_config
            .time
            .finish
            .sub(dist_config.applicable_start_time(env.block.time)),
    );
    if env.block.time.gt(&dist_config.time.start) {
        dist_config.reward_rate = if remove {
            Decimal256::from_uint256(
                dist_config
                    .reward_rate
                    .mul(remaining)
                    .sub(amount)
                    .div(Decimal256::from_uint256(remaining)),
            )
        } else {
            Decimal256::from_uint256(
                dist_config
                    .reward_rate
                    .mul(remaining)
                    .add(amount)
                    .div(Decimal256::from_uint256(remaining)),
            )
        };
    } else {
        dist_config.reward_rate = dist_config.reward_rate.add(Decimal256::from_uint256(
            amount.div(Decimal256::from_uint256(remaining)),
        ));
    }
    dist_config.total_reward_amount = dist_config.total_reward_amount.add(amount);

    config.distribution_config = dist_config;
    config::store(&mut deps.storage, &config).unwrap();

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", action),
            log("reward_rate_before", reward_rate_before),
            log("reward_rate_after", config.distribution_config.reward_rate),
        ],
        data: None,
    })
}

fn adjust_period<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    time: u64,
    shorten: bool,
) -> StdResult<HandleResponse> {
    let action = if shorten {
        "shorten_period"
    } else {
        "lengthen_period"
    };

    let mut config = config::read(&deps.storage).unwrap();
    let mut dist_config = config.distribution_config;
    validate_sender(&env, &config.owner, action)?;

    let reward_rate_before = dist_config.reward_rate;
    let finish_time_before = dist_config.time.finish;
    let remaining = Uint256::from(
        dist_config
            .time
            .finish
            .sub(dist_config.applicable_start_time(env.block.time)),
    );

    dist_config.reward_rate = if shorten {
        Decimal256::from_uint256(
            dist_config
                .reward_rate
                .mul(remaining)
                .div(Decimal256::from_uint256(remaining.sub(Uint256::from(time)))),
        )
    } else {
        Decimal256::from_uint256(
            dist_config
                .reward_rate
                .mul(remaining)
                .div(Decimal256::from_uint256(remaining.add(Uint256::from(time)))),
        )
    };

    dist_config.time.finish = dist_config.time.finish.add(time);
    config.distribution_config = dist_config;
    config::store(&mut deps.storage, &config).unwrap();

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", action),
            log("reward_rate_before", reward_rate_before),
            log("reward_rate_after", config.distribution_config.reward_rate),
            log("finish_time_before", finish_time_before),
            log("finish_time_after", config.distribution_config.time.finish),
        ],
        data: None,
    })
}
