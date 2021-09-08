use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{log, to_binary, CosmosMsg, HumanAddr, LogAttribute, StdError, WasmMsg};
use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdResult, Storage};
use cw20::Cw20HandleMsg;
use pylon_launchpad::lockup_msg::ConfigureMsg;
use std::cmp::{max, min};
use std::ops::{Add, Div, Mul, Sub};

use crate::handler::validator::{validate_config, validate_config_message, validate_sender};
use crate::lib_staking as staking;
use crate::state;

pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    target: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config: state::Config = state::read_config(&deps.storage)?;
    let applicable_reward_time = std::cmp::min(config.finish_time, env.block.time);

    // reward
    let mut reward: state::Reward = state::read_reward(&deps.storage)?;
    reward.reward_per_token_stored =
        reward
            .reward_per_token_stored
            .add(staking::calculate_reward_per_token(
                deps,
                &reward,
                &applicable_reward_time,
            )?);
    reward.last_update_time = applicable_reward_time;
    state::store_reward(&mut deps.storage, &reward)?;

    // user
    let mut user_update_logs = vec![];
    if let Some(target) = target {
        let t = deps.api.canonical_address(&target)?;

        let mut user: state::User = state::read_user(&deps.storage, &t)?;

        user.reward =
            staking::calculate_rewards(deps, &reward, &user, Option::from(applicable_reward_time))?;
        user.reward_per_token_paid = reward.reward_per_token_stored;

        state::store_user(&mut deps.storage, &t, &user)?;

        user_update_logs.append(&mut vec![log("target", target), log("reward", user.reward)]);
    }

    Ok(HandleResponse {
        messages: vec![],
        log: [
            vec![
                log("action", "update"),
                log("sender", env.message.sender),
                log("stored_rpt", reward.reward_per_token_stored),
            ],
            user_update_logs,
        ]
        .concat(),
        data: None,
    })
}

pub fn deposit_internal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    validate_sender(&env, &env.contract.address, "deposit_internal")?;
    let config: state::Config = state::read_config(&deps.storage)?;

    // check time range // open_deposit flag
    if env.block.time.lt(&config.start_time) && env.block.time.gt(&config.finish_time) {
        return Err(StdError::generic_err(format!(
            "Lockup: cannot deposit tokens out of period range. (now: {}, starts: {}, ends: {})",
            env.block.time, config.start_time, config.finish_time,
        )));
    }

    let owner = deps.api.canonical_address(&sender)?;
    let mut reward: state::Reward = state::read_reward(&deps.storage)?;
    let mut user: state::User = state::read_user(&deps.storage, &owner)?;

    reward.total_deposit = reward.total_deposit.add(amount);
    user.amount = user.amount.add(amount);

    state::store_reward(&mut deps.storage, &reward)?;
    state::store_user(&mut deps.storage, &owner, &user)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "deposit"),
            log("sender", env.message.sender),
            log("deposit_amount", amount),
        ],
        data: None,
    })
}

pub fn withdraw_internal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    validate_sender(&env, &env.contract.address, "withdraw_internal")?;
    let config: state::Config = state::read_config(&deps.storage)?;

    let in_temp_period = env.block.time.gt(&config.temp_withdraw_start_time)
        && env.block.time.lt(&config.temp_withdraw_finish_time);
    if !in_temp_period && env.block.time.lt(&config.finish_time) {
        return Err(StdError::generic_err(format!(
                "Lockup: cannot withdraw tokens during lockup period. (now: {}, temp_start: {}, temp_end: {}, ends: {})",
                env.block.time, config.temp_withdraw_start_time,config.temp_withdraw_finish_time, config.finish_time,
            )));
    }

    let owner = deps.api.canonical_address(&sender)?;
    let mut reward: state::Reward = state::read_reward(&deps.storage)?;
    let mut user: state::User = state::read_user(&deps.storage, &owner)?;

    if amount > user.amount {
        return Err(StdError::generic_err(
            "Staking: amount must be smaller than user.amount",
        ));
    }

    reward.total_deposit = reward.total_deposit.sub(amount);
    user.amount = user.amount.sub(amount);

    state::store_reward(&mut deps.storage, &reward)?;
    state::store_user(&mut deps.storage, &owner, &user)?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.share_token)?,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: sender.clone(),
                amount: amount.into(),
            })?,
            send: vec![],
        })],
        log: vec![
            log("action", "withdraw"),
            log("sender", sender),
            log("withdraw_amount", amount),
        ],
        data: None,
    })
}

pub fn claim_internal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
) -> StdResult<HandleResponse> {
    validate_sender(&env, &env.contract.address, "claim_internal")?;
    let config: state::Config = state::read_config(&deps.storage)?;

    // check time range // open_claim flag
    if env.block.time.lt(&config.cliff_time) {
        return Err(StdError::generic_err(format!(
            "Lockup: cannot claim rewards during lockup period. (now: {}, ends: {})",
            env.block.time, config.cliff_time
        )));
    }

    let owner = deps.api.canonical_address(&sender)?;
    let mut user: state::User = state::read_user(&deps.storage, &owner)?;

    let claim_amount = user.reward;
    user.reward = Uint256::zero();

    state::store_user(&mut deps.storage, &owner, &user)?;

    let config: state::Config = state::read_config(&deps.storage)?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&config.reward_token)?,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: sender.clone(),
                amount: claim_amount.into(),
            })?,
            send: vec![],
        })],
        log: vec![
            log("action", "claim"),
            log("sender", sender),
            log("claim_amount", claim_amount),
        ],
        data: None,
    })
}

pub fn configure<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: ConfigureMsg,
) -> StdResult<HandleResponse> {
    let mut config = state::read_config(&deps.storage).unwrap();
    validate_sender(
        &env,
        &deps.api.human_address(&config.owner).unwrap(),
        "add_reward",
    )?;

    // validate
    validate_config_message(&env, &msg)?;

    let remaining = Uint256::from(
        config
            .finish_time
            .sub(min(config.start_time, env.block.time)),
    );

    let mut logs: Vec<LogAttribute> = vec![log("action", "configure")];
    if let Some(owner) = msg.owner {
        config.owner = deps.api.canonical_address(&owner).unwrap();
        logs.push(log("new_owner", owner));
    }
    if let Some(start_time) = msg.start_time {
        config.start_time = start_time;
        logs.push(log("new_start_time", start_time));
    }
    if let Some(cliff_time) = msg.cliff_time {
        config.cliff_time = cliff_time;
        logs.push(log("new_cliff_time", cliff_time));
    }
    if let Some(finish_time) = msg.finish_time {
        config.finish_time = finish_time;
        config.reward_rate = Decimal256::from_uint256(config.reward_rate.mul(remaining).div(
            Decimal256::from_uint256(Uint256::from(finish_time.sub(env.block.time))),
        ));
        logs.push(log("new_finish_time", finish_time));
    }
    if let Some(temp_withdraw_start_time) = msg.temp_withdraw_start_time {
        config.temp_withdraw_start_time = temp_withdraw_start_time;
        logs.push(log(
            "new_temp_withdraw_start_time",
            temp_withdraw_start_time,
        ));
    }
    if let Some(temp_withdraw_finish_time) = msg.temp_withdraw_finish_time {
        config.temp_withdraw_finish_time = temp_withdraw_finish_time;
        logs.push(log(
            "new_temp_withdraw_finish_time",
            temp_withdraw_finish_time,
        ));
    }

    // validate
    validate_config(&config)?;

    // store
    state::store_config(&mut deps.storage, &config).unwrap();

    Ok(HandleResponse {
        messages: vec![],
        log: logs,
        data: None,
    })
}

pub fn add_reward<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    let mut config = state::read_config(&deps.storage).unwrap();
    if config
        .owner
        .ne(&deps.api.canonical_address(&env.message.sender).unwrap())
    {
        return Err(StdError::generic_err(format!(
            "Lockup: cannot execute add_reward message with unauthorized sender. expected: {}, actual: {}",
            deps.api.human_address(&config.owner).unwrap(), env.message.sender,
        )));
    }

    let reward_rate_before = config.reward_rate;
    let remaining = Uint256::from(
        config
            .finish_time
            .sub(max(config.start_time, env.block.time)),
    );
    if env.block.time.gt(&config.start_time) {
        config.reward_rate = Decimal256::from_uint256(
            config
                .reward_rate
                .mul(remaining)
                .add(amount)
                .div(Decimal256::from_uint256(remaining)),
        );
    } else {
        config.reward_rate = config.reward_rate.add(Decimal256::from_uint256(
            amount.div(Decimal256::from_uint256(remaining)),
        ));
    }

    state::store_config(&mut deps.storage, &config).unwrap();

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "add_reward"),
            log("reward_rate_before", reward_rate_before),
            log("reward_rate_after", config.reward_rate),
        ],
        data: None,
    })
}

pub fn sub_reward<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    let mut config = state::read_config(&deps.storage).unwrap();
    if config
        .owner
        .ne(&deps.api.canonical_address(&env.message.sender).unwrap())
    {
        return Err(StdError::generic_err(format!(
            "Lockup: cannot execute sub_reward message with unauthorized sender. expected: {}, actual: {}",
            deps.api.human_address(&config.owner).unwrap(), env.message.sender,
        )));
    }
    if env.block.time.gt(&config.finish_time) {
        return Err(StdError::generic_err(format!(
            "Lockup: sale finished. execution_time: {}, finish_time: {}",
            env.block.time, config.finish_time,
        )));
    }

    let reward_rate_before = config.reward_rate;
    let remaining = Uint256::from(
        config
            .finish_time
            .sub(max(config.start_time, env.block.time)),
    );
    if env.block.time.gt(&config.start_time) {
        config.reward_rate = Decimal256::from_uint256(
            config
                .reward_rate
                .mul(remaining)
                .sub(amount)
                .div(Decimal256::from_uint256(remaining)),
        );
    } else {
        config.reward_rate = config.reward_rate.sub(Decimal256::from_uint256(
            amount.div(Decimal256::from_uint256(remaining)),
        ));
    }

    state::store_config(&mut deps.storage, &config).unwrap();

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "sub_reward"),
            log("reward_rate_before", reward_rate_before),
            log("reward_rate_after", config.reward_rate),
        ],
        data: None,
    })
}
