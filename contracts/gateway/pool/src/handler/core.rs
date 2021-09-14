use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{log, to_binary, CosmosMsg, HumanAddr, StdError, WasmMsg};
use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdResult, Storage};
use cw20::Cw20HandleMsg;
use std::ops::{Add, Sub};

use crate::handler::{util_staking, validate_sender};
use crate::state::{config, reward, user};

pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    target: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config = config::read(&deps.storage).unwrap().distribution_config;
    let applicable_reward_time = config.applicable_reward_time(env.block.time);

    // reward
    let mut reward = reward::read(&deps.storage).unwrap();
    reward.reward_per_token_stored =
        reward
            .reward_per_token_stored
            .add(util_staking::calculate_reward_per_token(
                deps,
                &reward,
                applicable_reward_time,
            )?);
    reward.last_update_time = applicable_reward_time;
    reward::store(&mut deps.storage, &reward).unwrap();

    // user
    let mut user_update_logs = vec![];
    if let Some(target) = target {
        let t = deps.api.canonical_address(&target).unwrap();

        let mut user = user::read(&deps.storage, &t).unwrap();

        user.reward = util_staking::calculate_rewards(
            deps,
            &reward,
            &user,
            Option::from(applicable_reward_time),
        )?;
        user.reward_per_token_paid = reward.reward_per_token_stored;

        user::store(&mut deps.storage, &t, &user).unwrap();

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

    let config = config::read(&deps.storage).unwrap();
    // check deposit config & temporary deposit config
    config.check_deposit_time(&env)?;

    let owner = deps.api.canonical_address(&sender).unwrap();
    let mut reward = reward::read(&deps.storage).unwrap();
    let mut user = user::read(&deps.storage, &owner).unwrap();

    // check total cap
    reward.total_deposit = reward.total_deposit.add(amount);
    if config.deposit_config.total_cap.ne(&Uint256::zero())
        && reward.total_deposit.gt(&config.deposit_config.total_cap)
    {
        return Err(StdError::generic_err(format!(
            "Lockup: deposit amount exceeds total cap. cap: {}",
            config.deposit_config.total_cap,
        )));
    }

    // check user cap
    user.amount = user.amount.add(amount);
    if config.deposit_config.user_cap.ne(&Uint256::zero())
        && user.amount.gt(&config.deposit_config.user_cap)
    {
        return Err(StdError::generic_err(format!(
            "Lockup: deposit amount exceeds user cap. cap: {}",
            config.deposit_config.user_cap,
        )));
    }

    reward::store(&mut deps.storage, &reward).unwrap();
    user::store(&mut deps.storage, &owner, &user).unwrap();

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

    let config = config::read(&deps.storage).unwrap();
    // check withdraw config & temporary withdraw config
    config.check_withdraw_time(&env)?;

    let owner = deps.api.canonical_address(&sender).unwrap();
    let mut reward = reward::read(&deps.storage).unwrap();
    let mut user = user::read(&deps.storage, &owner).unwrap();

    if amount > user.amount {
        return Err(StdError::generic_err(
            "Staking: amount must be smaller than user.amount",
        ));
    }

    reward.total_deposit = reward.total_deposit.sub(amount);
    user.amount = user.amount.sub(amount);

    reward::store(&mut deps.storage, &reward).unwrap();
    user::store(&mut deps.storage, &owner, &user).unwrap();

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.share_token,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: sender.clone(),
                amount: amount.into(),
            })
            .unwrap(),
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
    let config = config::read(&deps.storage).unwrap();
    // check claim config
    config.check_claim_time(&env)?;

    let owner = deps.api.canonical_address(&sender).unwrap();
    let mut user = user::read(&deps.storage, &owner).unwrap();

    let claim_amount = user.reward;
    user.reward = Uint256::zero();
    user::store(&mut deps.storage, &owner, &user).unwrap();

    let config = config::read(&deps.storage).unwrap();

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.reward_token,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: sender.clone(),
                amount: claim_amount.into(),
            })
            .unwrap(),
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
