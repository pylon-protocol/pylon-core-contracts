use crate::handler::validate_sender;
use crate::state::config;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{log, Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage};
use pylon_launchpad::lockup_msg::{ConfigureMsg, DistributionMsg};
use std::ops::{Add, Div, Mul, Sub};

pub fn configure<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: ConfigureMsg,
) -> StdResult<HandleResponse> {
    match msg {
        // TODO: handle more configure messages
        ConfigureMsg::Distribution(msg) => configure_distribution(deps, env, msg),
        _ => Err(StdError::NotFound {
            kind: "".to_string(),
            backtrace: None,
        }),
    }
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
            .sub(dist_config.applicable_start_time(&env)),
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
            .sub(dist_config.applicable_start_time(&env)),
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
