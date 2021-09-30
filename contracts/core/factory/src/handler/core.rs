use cosmwasm_std::*;
use pylon_core::factory_msg::ConfigureMsg;
use pylon_core::pool_v2_msg::InstantiateMsg;
use std::ops::Add;

use crate::error::ContractError;
use crate::state::{adapter, config, pool, state};

pub fn configure(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ConfigureMsg,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage)?;
    if config.owner.ne(&info.sender.to_string()) {
        return Err(ContractError::Unauthorized {
            action: "Factory/configure".to_string(),
            expected: config.owner,
            actual: info.sender.to_string(),
        });
    }

    let mut attrs: Vec<Attribute> = vec![
        Attribute::new("action", "configure"),
        Attribute::new("sender", info.sender.to_string()),
    ];
    if let Some(o) = msg.owner {
        config.owner = o.clone();
        attrs.push(Attribute::new("new_owner".to_string(), o));
    }
    if let Some(p) = msg.pool_code_id {
        config.pool_code_id = p;
        attrs.push(Attribute::new("new_pid".to_string(), p.to_string()));
    }
    if let Some(t) = msg.token_code_id {
        config.token_code_id = t;
        attrs.push(Attribute::new("new_tid".to_string(), t.to_string()));
    }
    if let Some(f) = msg.fee_rate {
        config.fee_rate = f;
        attrs.push(Attribute::new("new_fee_rate".to_string(), f.to_string()));
    }
    if let Some(f) = msg.fee_collector {
        config.fee_collector = f.clone();
        attrs.push(Attribute::new("new_fee_collector".to_string(), f));
    }

    config::store(deps.storage, &config)?;

    Ok(Response::new().add_attributes(attrs))
}

pub fn create_pool(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pool_name: String,
    beneficiary: String,
    yield_adapter: String,
) -> Result<Response, ContractError> {
    let adapter = adapter::read(deps.storage, yield_adapter.clone())?;
    if adapter.address.is_empty() {
        return Err(ContractError::Std(StdError::generic_err(
            "Factory: given yield adapter not allowed",
        )));
    }

    let mut state = state::read(deps.storage)?;
    let mut pool = pool::read(deps.storage, state.next_pool_id)?;

    pool.id = state.next_pool_id;
    pool.status = pool::Status::Ready;
    pool::store(deps.storage, pool.id, &pool)?;

    state.next_pool_id = state.next_pool_id.add(1);
    state::store(deps.storage, &state)?;

    let config = config::read(deps.storage)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id: config.pool_code_id,
            msg: to_binary(&InstantiateMsg {
                pool_id: pool.id,
                pool_name,
                beneficiary,
                yield_adapter,
                dp_code_id: config.token_code_id,
            })
            .unwrap(),
            funds: vec![],
            label: "".to_string(),
        }))
        .add_attribute("action", "create_pool")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("pool_id", pool.id.to_string()))
}

pub fn register_pool(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    pool_id: u64,
) -> Result<Response, ContractError> {
    let mut pool = pool::read(deps.storage, pool_id)?;
    if pool.status.ne(&pool::Status::Ready) {
        return Err(ContractError::Std(StdError::generic_err(
            "Factory: pool is not on ready status",
        )));
    }

    pool.status = pool::Status::Deployed;
    pool.address = info.sender.to_string();
    pool::store(deps.storage, pool.id, &pool)?;

    Ok(Response::new()
        .add_attribute("action", "register_pool")
        .add_attribute("sender", info.sender.to_string()))
}

pub fn register_adapter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    adapter: String,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;
    if config.owner.ne(&info.sender.to_string()) {
        return Err(ContractError::Unauthorized {
            action: "Factory/register_adapter".to_string(),
            expected: config.owner,
            actual: info.sender.to_string(),
        });
    }

    adapter::store(
        deps.storage,
        adapter.clone(),
        &adapter::Adapter {
            address: adapter.clone(),
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "register_adapter")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("adapter", adapter))
}

pub fn unregister_adapter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    adapter: String,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage)?;
    if config.owner.ne(&info.sender.to_string()) {
        return Err(ContractError::Unauthorized {
            action: "Factory/unregister_adapter".to_string(),
            expected: config.owner,
            actual: info.sender.to_string(),
        });
    }

    adapter::remove(deps.storage, adapter.clone());

    Ok(Response::new()
        .add_attribute("action", "unregister_adapter")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("adapter", adapter))
}
