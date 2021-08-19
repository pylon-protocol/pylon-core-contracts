use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    log, to_binary, Api, CanonicalAddr, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    LogAttribute, Querier, StdError, StdResult, Storage, WasmMsg,
};
use pylon_core::pool_msg::InitMsg;
use std::ops::Add;

use crate::state::{adapter, config, pool, state};

pub fn configure<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    pool_code_id: Option<u64>,
    token_code_id: Option<u64>,
    fee_rate: Option<Decimal256>,
    fee_collector: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    let mut config = config::read(&deps.storage)?;
    let sender = deps.api.canonical_address(&env.message.sender)?;
    if config.owner.ne(&sender) {
        return Err(StdError::unauthorized());
    }

    let mut logs: Vec<LogAttribute> = vec![
        log("action", "configure"),
        log("sender", env.message.sender),
    ];
    if let Some(o) = owner {
        config.owner = deps.api.canonical_address(&o)?;
        logs.push(log("new_owner", o));
    }
    if let Some(p) = pool_code_id {
        config.pool_code_id = p.clone();
        logs.push(log("new_pid", p));
    }
    if let Some(t) = token_code_id {
        config.token_code_id = t.clone();
        logs.push(log("new_tid", t));
    }
    if let Some(f) = fee_rate {
        config.fee_rate = f.clone();
        logs.push(log("new_fee_rate", f));
    }
    if let Some(f) = fee_collector {
        config.fee_collector = deps.api.canonical_address(&f)?;
        logs.push(log("new_fee_collector", f));
    }

    config::store(&mut deps.storage, &config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: logs,
        data: None,
    })
}

pub fn create_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pool_name: String,
    beneficiary: HumanAddr,
    yield_adapter: HumanAddr,
) -> StdResult<HandleResponse> {
    let adapter = adapter::read(&deps.storage, deps.api.canonical_address(&yield_adapter)?)?;
    if adapter.address.is_empty() {
        return Err(StdError::generic_err(
            "Factory: given yield adapter not allowed",
        ));
    }

    let mut state = state::read(&deps.storage)?;
    let mut pool = pool::read(&deps.storage, state.next_pool_id)?;

    pool.id = state.next_pool_id.clone();
    pool.status = pool::Status::Ready;
    pool.address = CanonicalAddr::default();
    pool::store(&mut deps.storage, pool.id, &pool)?;

    state.next_pool_id = state.next_pool_id.add(1);
    state::store(&mut deps.storage, &state)?;

    let config = config::read(&deps.storage)?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: config.pool_code_id.clone(),
            send: vec![],
            label: None,
            msg: to_binary(&InitMsg {
                pool_id: pool.id,
                pool_name,
                beneficiary,
                fee_collector: deps.api.human_address(&config.fee_collector)?,
                yield_adapter,
                dp_code_id: config.token_code_id.clone(),
            })?,
        })],
        log: vec![
            log("action", "create_pool"),
            log("sender", env.message.sender),
            log("pool_id", pool.id),
        ],
        data: None,
    })
}

pub fn register_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pool_id: u64,
) -> StdResult<HandleResponse> {
    let mut pool = pool::read(&deps.storage, pool_id)?;
    if pool.status.ne(&pool::Status::Ready) {
        return Err(StdError::generic_err(
            "Factory: pool is not on ready status",
        ));
    }

    pool.status = pool::Status::Deployed;
    pool.address = deps.api.canonical_address(&env.message.sender)?;
    pool::store(&mut deps.storage, pool.id, &pool)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "register_pool"),
            log("sender", env.message.sender),
        ],
        data: None,
    })
}

pub fn register_adapter<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    adapter: HumanAddr,
) -> StdResult<HandleResponse> {
    let config = config::read(&deps.storage)?;
    if config
        .owner
        .ne(&deps.api.canonical_address(&env.message.sender)?)
    {
        return Err(StdError::generic_err("Factory: only owner (register)"));
    }

    let address = deps.api.canonical_address(&adapter)?;
    adapter::store(
        &mut deps.storage,
        address.clone(),
        &adapter::Adapter { address },
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "register_adapter"),
            log("sender", env.message.sender),
            log("adapter", adapter),
        ],
        data: None,
    })
}

pub fn unregister_adapter<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    adapter: HumanAddr,
) -> StdResult<HandleResponse> {
    let config = config::read(&deps.storage)?;
    if config
        .owner
        .ne(&deps.api.canonical_address(&env.message.sender)?)
    {
        return Err(StdError::generic_err("Factory: only owner (register)"));
    }

    let address = deps.api.canonical_address(&adapter)?;
    adapter::remove(&mut deps.storage, address);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "unregister_adapter"),
            log("sender", env.message.sender),
            log("adapter", adapter),
        ],
        data: None,
    })
}
