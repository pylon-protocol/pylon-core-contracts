use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult,
    Querier, StdResult, Storage,
};
use pylon_core::exchange_rate_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};

use crate::handler_exec as ExecHandler;
use crate::handler_query as QueryHandler;
use crate::state;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _: InitMsg,
) -> StdResult<InitResponse> {
    state::store_config(
        &mut deps.storage,
        &state::Config {
            owner: env.message.sender,
        },
    )?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Update { token } => Ok(ExecHandler::update(deps, env, token)?),
        HandleMsg::ConfigToken {
            token,
            exchange_rate,
            epoch_period,
            weight,
        } => Ok(ExecHandler::config_token(
            deps,
            env,
            token,
            exchange_rate,
            epoch_period,
            weight,
        )?),
        HandleMsg::AddToken {
            token,
            base_rate,
            period,
            weight,
        } => Ok(ExecHandler::add_token(
            deps, env, token, base_rate, period, weight,
        )?),
        HandleMsg::Start { tokens } => Ok(ExecHandler::start(deps, env, tokens)?),
        HandleMsg::Stop { tokens } => Ok(ExecHandler::stop(deps, env, tokens)?),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::ExchangeRateOf { token, blocktime } => {
            Ok(QueryHandler::exchange_rate_of(deps, &token, blocktime)?)
        }
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _: &mut Extern<S, A, Q>,
    _: Env,
    _: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
