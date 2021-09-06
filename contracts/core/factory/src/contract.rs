use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult,
    Querier, StdResult, Storage,
};
use pylon_core::factory_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};

use crate::handler::{core as CoreHandler, query as QueryHandler};
use crate::state::{config, state};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = config::Config {
        owner: deps.api.canonical_address(&env.message.sender)?,
        pool_code_id: msg.pool_code_id,
        token_code_id: msg.token_code_id,
        fee_rate: msg.fee_rate,
        fee_collector: deps.api.canonical_address(&msg.fee_collector)?,
    };

    let state = state::State { next_pool_id: 0 };

    config::store(&mut deps.storage, &config)?;
    state::store(&mut deps.storage, &state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Configure {
            owner,
            pool_code_id,
            token_code_id,
            fee_rate,
            fee_collector,
        } => CoreHandler::configure(
            deps,
            env,
            owner,
            pool_code_id,
            token_code_id,
            fee_rate,
            fee_collector,
        ),
        HandleMsg::CreatePool {
            pool_name,
            beneficiary,
            yield_adapter,
        } => CoreHandler::create_pool(deps, env, pool_name, beneficiary, yield_adapter),
        HandleMsg::RegisterPool { pool_id } => CoreHandler::register_pool(deps, env, pool_id),
        HandleMsg::RegisterAdapter { address } => CoreHandler::register_adapter(deps, env, address),
        HandleMsg::UnregisterAdapter { address } => {
            CoreHandler::unregister_adapter(deps, env, address)
        }
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::PoolInfo { pool_id } => QueryHandler::pool_info(deps, pool_id),
        QueryMsg::PoolInfos { start_after, limit } => {
            QueryHandler::pool_infos(deps, start_after, limit)
        }
        QueryMsg::AdapterInfo { address } => QueryHandler::adapter_info(deps, address),
        QueryMsg::AdapterInfos { start_after, limit } => QueryHandler::adapter_infos(
            deps,
            match start_after {
                Some(start_after) => {
                    Option::from(deps.api.canonical_address(&start_after).unwrap())
                }
                None => Option::None,
            },
            limit,
        ),
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
