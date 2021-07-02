use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    log, Api, CanonicalAddr, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult,
    Storage,
};
use std::ops::{Div, Mul, Sub};

use crate::state;

pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token: HumanAddr,
) -> StdResult<HandleResponse> {
    let token_addr: CanonicalAddr = deps.api.canonical_address(&token)?;

    let mut token: state::Token = state::read_token(&deps.storage, &token_addr)?;
    if token.status.ne(&state::Status::Running) {
        return Err(StdError::generic_err("Feeder: invalid token status"));
    }

    let elapsed = env.block.time.sub(token.last_updated_at);
    if elapsed < token.epoch_period {
        return Ok(HandleResponse::default());
    }

    let exchange_rate_before = token.exchange_rate;
    let pow_count = elapsed.div(token.epoch_period);
    for _ in 0..pow_count {
        token.exchange_rate = token.exchange_rate.mul(token.weight);
    }

    token.last_updated_at = env.block.time;

    state::store_token(&mut deps.storage, &token_addr, &token)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "update"),
            log("sender", env.message.sender),
            log("er_before", exchange_rate_before),
            log("er_after", token.exchange_rate),
        ],
        data: None,
    })
}

fn check_owner<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    caller: &HumanAddr,
) -> StdResult<()> {
    let config: state::Config = state::read_config(&deps.storage)?;
    if config.owner.ne(caller) {
        return Err(StdError::generic_err(format!(
            "Feeder: only owner can execute this function. (owner: {}, sender: {})",
            config.owner, caller,
        )));
    }

    Ok(())
}

fn _stop<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    token: HumanAddr,
) -> StdResult<()> {
    let token_addr: CanonicalAddr = deps.api.canonical_address(&token)?;
    let mut token: state::Token = state::read_token(&deps.storage, &token_addr)?;

    token.status = state::Status::Stopped;

    state::store_token(&mut deps.storage, &token_addr, &token)?;

    Ok(())
}

pub fn stop<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    tokens: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    check_owner(deps, &env.message.sender)?;
    for token in tokens {
        _stop(deps, token)?;
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "stop"), log("sender", env.message.sender)],
        data: None,
    })
}

fn _start<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    token: &HumanAddr,
    block_time: &u64,
) -> StdResult<()> {
    let token_addr: CanonicalAddr = deps.api.canonical_address(&token)?;
    let mut token: state::Token = state::read_token(&deps.storage, &token_addr)?;

    token.status = state::Status::Running;
    token.last_updated_at = *block_time;

    state::store_token(&mut deps.storage, &token_addr, &token)?;

    Ok(())
}

pub fn start<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    tokens: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    check_owner(deps, &env.message.sender)?;
    for token in tokens {
        _start(deps, &token, &env.block.time)?;
    }
    Ok(HandleResponse::default())
}

pub fn config_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token: HumanAddr,
    exchange_rate: Option<Decimal256>,
    epoch_period: Option<u64>,
    weight: Option<Decimal256>,
) -> StdResult<HandleResponse> {
    check_owner(deps, &env.message.sender)?;

    let token_addr: CanonicalAddr = deps.api.canonical_address(&token)?;
    let mut token: state::Token = state::read_token(&deps.storage, &token_addr)?;

    if let Some(er) = exchange_rate {
        if token.exchange_rate.gt(&er) {
            return Err(StdError::unauthorized());
        }
        token.exchange_rate = er;
    }
    if let Some(ep) = epoch_period {
        token.epoch_period = ep;
    }
    if let Some(w) = weight {
        token.weight = w;
    }

    state::store_token(&mut deps.storage, &token_addr, &token)?;

    Ok(HandleResponse::default())
}

pub fn add_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    token: HumanAddr,
    base_rate: Decimal256,
    period: u64,
    weight: Decimal256,
) -> StdResult<HandleResponse> {
    check_owner(deps, &env.message.sender)?;

    let token_addr: CanonicalAddr = deps.api.canonical_address(&token)?;

    state::store_token(
        &mut deps.storage,
        &token_addr,
        &state::Token {
            exchange_rate: base_rate,
            epoch_period: period,
            status: state::Status::Neutral,
            weight,
            last_updated_at: env.block.time,
        },
    )?;

    Ok(HandleResponse::default())
}
