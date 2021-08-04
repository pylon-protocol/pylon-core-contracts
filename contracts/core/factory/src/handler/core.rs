use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    log, to_binary, Api, CanonicalAddr, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier,
    StdError, StdResult, Storage, WasmMsg,
};
use pylon_core::pool_msg::InitMsg;
use std::ops::Add;

use crate::state::{adapter, config, pool, state};

pub fn configure<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
    pool_code_id: u64,
    token_code_id: u64,
    fee_collector: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config = config::read(&deps.storage)?;
    let sender = deps.api.canonical_address(&env.message.sender)?;
    if config.owner.ne(&sender) {
        return Err(StdError::unauthorized());
    }

    config.owner = deps.api.canonical_address(&owner)?;
    config.pool_code_id = pool_code_id;
    config.token_code_id = token_code_id;
    config.fee_collector = deps.api.canonical_address(&fee_collector)?;
    config::store(&mut deps.storage, &config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "configure"),
            log("sender", env.message.sender),
        ],
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
    fee_rate: Decimal256,
) -> StdResult<HandleResponse> {
    let address = deps.api.canonical_address(&adapter)?;
    adapter::store(
        &mut deps.storage,
        address.clone(),
        &adapter::Adapter {
            address,
            fee_rate: fee_rate.clone(),
        },
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "register_adapter"),
            log("sender", env.message.sender),
            log("adapter", adapter),
            log("fee_rate", fee_rate),
        ],
        data: None,
    })
}

pub fn unregister_adapter<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    adapter: HumanAddr,
) -> StdResult<HandleResponse> {
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
