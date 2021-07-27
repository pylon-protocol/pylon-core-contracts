use cosmwasm_bignumber::Uint256;
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
        pool_code_id: msg.pool_code_id.clone(),
        token_code_id: msg.token_code_id.clone(),
        fee_collector: deps.api.canonical_address(&msg.fee_collector)?,
    };

    let state = state::State {
        next_pool_id: Uint256::zero(),
    };

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
            fee_collector,
        } => CoreHandler::configure(deps, env, owner, pool_code_id, token_code_id, fee_collector),
        HandleMsg::CreatePool {
            pool_name,
            beneficiary,
            yield_adapter,
        } => CoreHandler::create_pool(deps, env, pool_name, beneficiary, yield_adapter),
        HandleMsg::RegisterPool { pool_id } => CoreHandler::register_pool(deps, env, pool_id),
        HandleMsg::RegisterAdapter { address, fee_rate } => {
            CoreHandler::register_adapter(deps, env, address, fee_rate)
        }
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
        QueryMsg::State {} => QueryHandler::state(deps),
        QueryMsg::PoolInfo { pool_id } => QueryHandler::pool_info(deps, pool_id),
        QueryMsg::AdapterInfo { address } => QueryHandler::adapter_info(deps, address),
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
