use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{from_binary, log, to_binary, CosmosMsg, HumanAddr, StdError, WasmMsg};
use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdResult, Storage};
use cw20::{Cw20HandleMsg, Cw20ReceiveMsg};
use std::ops::{Add, Sub};

use pylon_launchpad::lockup_msg::Cw20HookMsg;

use crate::lib_staking as staking;
use crate::state;

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender.clone();

    if let Some(msg) = cw20_msg.msg {
        match from_binary(&msg)? {
            Cw20HookMsg::Deposit {} => {
                let config: state::Config = state::read_config(&deps.storage)?;
                if deps.api.canonical_address(&sender)? != config.share_token {
                    return Err(StdError::unauthorized());
                }

                deposit(deps, env, cw20_msg.sender, Uint256::from(cw20_msg.amount))
            }
        }
    } else {
        Err(StdError::generic_err("Staking: unsupported message"))
    }
}

pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    target: Option<&HumanAddr>,
) -> StdResult<HandleResponse> {
    let config: state::Config = state::read_config(&deps.storage)?;
    let mut reward: state::Reward = state::read_reward(&deps.storage)?;

    let applicable_reward_time = if env.block.time.gt(&config.finish_time) {
        config.finish_time
    } else {
        env.block.time
    };

    // reward
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
    if let Some(target) = target {
        let t = deps.api.canonical_address(target)?;

        let mut user: state::User = state::read_user(&deps.storage, &t)?;

        user.reward =
            staking::calculate_rewards(deps, &reward, &user, Option::from(applicable_reward_time))?;
        user.reward_per_token_paid = reward.reward_per_token_stored;

        state::store_user(&mut deps.storage, &t, &user)?;
    }

    Ok(HandleResponse::default())
}

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    let config: state::Config = state::read_config(&deps.storage)?;

    // check time range // open_deposit flag
    if env.block.time.gt(&config.start_time) && env.block.time.lt(&config.finish_time) {
        if !config.open_deposit {
            return Err(StdError::unauthorized());
        }
    }

    update(deps, &env, Option::from(&sender))?;

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
            log("sender", sender.clone()),
            log("deposit_amount", amount.clone()),
        ],
        data: None,
    })
}

pub fn withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    let config: state::Config = state::read_config(&deps.storage)?;

    // check time range // open_withdraw flag
    if env.block.time.gt(&config.start_time) && env.block.time.lt(&config.finish_time) {
        if !config.open_withdraw {
            return Err(StdError::unauthorized());
        }
    }

    let sender = &env.message.sender;
    update(deps, &env, Option::from(sender))?;

    let owner = deps.api.canonical_address(sender)?;
    let mut reward: state::Reward = state::read_reward(&deps.storage)?;
    let mut user: state::User = state::read_user(&deps.storage, &owner)?;

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

pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config: state::Config = state::read_config(&deps.storage)?;

    // check time range // open_claim flag
    if env.block.time.gt(&config.start_time) && env.block.time.lt(&config.finish_time) {
        if !config.open_claim {
            return Err(StdError::unauthorized());
        }
    }

    let sender = &env.message.sender;
    update(deps, &env, Option::from(sender))?;

    let owner = deps.api.canonical_address(sender)?;
    let mut user: state::User = state::read_user(&deps.storage, &owner)?;

    let claim_amount = user.reward.clone();
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
            log("sender", sender.clone()),
            log("claim_amount", claim_amount.clone()),
        ],
        data: None,
    })
}

pub fn exit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config: state::Config = state::read_config(&deps.storage)?;

    // check time range // open_claim flag
    if env.block.time.gt(&config.start_time) && env.block.time.lt(&config.finish_time) {
        if !config.open_withdraw || !config.open_claim {
            return Err(StdError::unauthorized());
        }
    }

    let sender = &env.message.sender;
    update(deps, &env, Option::from(sender))?;

    let owner = deps.api.canonical_address(sender)?;
    let mut reward: state::Reward = state::read_reward(&deps.storage)?;
    let mut user: state::User = state::read_user(&deps.storage, &owner)?;

    let withdraw_amount = user.amount.clone();
    user.amount = Uint256::zero();

    let claim_amount = user.reward.clone();
    user.reward = Uint256::zero();

    reward.total_deposit = reward.total_deposit.sub(withdraw_amount);

    state::store_user(&mut deps.storage, &owner, &user)?;
    state::store_reward(&mut deps.storage, &reward)?;

    let config: state::Config = state::read_config(&deps.storage)?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.share_token)?,
                msg: to_binary(&Cw20HandleMsg::Transfer {
                    recipient: sender.clone(),
                    amount: withdraw_amount.into(),
                })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.reward_token)?,
                msg: to_binary(&Cw20HandleMsg::Transfer {
                    recipient: sender.clone(),
                    amount: claim_amount.into(),
                })?,
                send: vec![],
            }),
        ],
        log: vec![
            log("action", "exit"),
            log("sender", sender.clone()),
            log("claim_amount", claim_amount.clone()),
            log("withdraw_amount", withdraw_amount.clone()),
        ],
        data: None,
    })
}
