use cosmwasm_std::{from_binary, HumanAddr, StdError, Uint128};
use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdResult, Storage};

use cw20::Cw20ReceiveMsg;

use crate::config;
use crate::msg::Cw20HookMsg;
use crate::state;
use cosmwasm_bignumber::Decimal256;
use std::ops::Add;

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender.clone();

    if let Some(msg) = cw20_msg.msg {
        match from_binary(&msg)? {
            Cw20HookMsg::Deposit {} => {
                let config: config::Config = config::read(&deps.storage)?;
                if deps.api.canonical_address(&sender)? != config.dp_token {
                    return Err(StdError::unauthorized());
                }
                if env.block.time.gt(&config.start_time) && !config.open_deposit {
                    return Err(StdError::unauthorized());
                }

                deposit(deps, env, cw20_msg.sender, cw20_msg.amount)
            }
            Cw20HookMsg::Configure {
                start_time,
                period,
                open_deposit,
                open_withdraw,
                open_claim,
                reward_rate,
            } => {
                let config: config::Config = config::read(&deps.storage)?;
                if deps.api.canonical_address(&sender)? != config.reward_token {
                    return Err(StdError::unauthorized());
                }
                if env.block.time.gt(&config.start_time) {
                    return Err(StdError::unauthorized());
                }
                if cw20_msg.amount < Uint256::from(period) * reward_rate {
                    return Err(StdError::generic_err("Staking: insufficient amount"));
                }

                configure(
                    deps,
                    env,
                    start_time,
                    period,
                    open_deposit,
                    open_withdraw,
                    open_claim,
                    reward_rate,
                )
            }
        }
    } else {
        Err(StdError::generic_err("Staking: unsupported message"))
    }
}

pub fn configure<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    start_time: u64,
    period: u64,
    open_deposit: bool,
    open_withdraw: bool,
    open_claim: bool,
    reward_rate: Decimal256,
) -> StdResult<HandleResponse> {
    let config: config::Config = config::read(&deps.storage)?;

    config::store(
        &mut deps.storage,
        &config::Config {
            owner: config.owner.clone(),
            dp_token: config.dp_token.clone(),
            reward_token: config.reward_token.clone(),
            start_time: start_time.clone(),
            finish_time: start_time.add(period),
            open_deposit,
            open_withdraw,
            open_claim,
            reward_rate,
        },
    )?;

    Ok(HandleResponse::default())
}

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config: config::Config = config::read(&deps.storage)?;
    let state: state::State = state::read(&deps.storage)?;

    // TODO

    Ok(HandleResponse::default())
}

pub fn withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config: config::Config = config::read(&deps.storage)?;
    let state: state::State = state::read(&deps.storage)?;

    // TODO

    Ok(HandleResponse::default())
}

pub fn exit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config: config::Config = config::read(&deps.storage)?;
    let state: state::State = state::read(&deps.storage)?;

    // TODO

    Ok(HandleResponse::default())
}

pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config: config::Config = config::read(&deps.storage)?;
    let state: state::State = state::read(&deps.storage)?;

    // TODO

    Ok(HandleResponse::default())
}
