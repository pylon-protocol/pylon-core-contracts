#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError};
use pylon_core::factory_msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::error::ContractError;
use crate::handler::{core as CoreHandler, query as QueryHandler};
use crate::state::{config, state};

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = config::Config {
        owner: info.sender.to_string(),
        pool_code_id: msg.pool_code_id,
        token_code_id: msg.token_code_id,
        fee_rate: msg.fee_rate,
        fee_collector: msg.fee_collector,
    };
    let state = state::State { next_pool_id: 0 };

    config::store(deps.storage, &config)?;
    state::store(deps.storage, &state)?;

    Ok(Response::default())
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Configure(msg) => CoreHandler::configure(deps, env, info, msg),
        ExecuteMsg::CreatePool {
            pool_name,
            beneficiary,
            yield_adapter,
        } => CoreHandler::create_pool(deps, env, info, pool_name, beneficiary, yield_adapter),
        ExecuteMsg::RegisterPool { pool_id } => {
            CoreHandler::register_pool(deps, env, info, pool_id)
        }
        ExecuteMsg::RegisterAdapter { address } => {
            CoreHandler::register_adapter(deps, env, info, address)
        }
        ExecuteMsg::UnregisterAdapter { address } => {
            CoreHandler::unregister_adapter(deps, env, info, address)
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, StdError> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::PoolInfo { pool_id } => QueryHandler::pool_info(deps, pool_id),
        QueryMsg::PoolInfos { start_after, limit } => {
            QueryHandler::pool_infos(deps, start_after, limit)
        }
        QueryMsg::AdapterInfo { address } => QueryHandler::adapter_info(deps, address),
        QueryMsg::AdapterInfos { start_after, limit } => {
            QueryHandler::adapter_infos(deps, start_after, limit)
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
