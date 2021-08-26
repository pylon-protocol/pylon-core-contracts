use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, MigrateResponse,
    MigrateResult, Querier, StdResult, Storage,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::handler::execute as ExecHandler;
use crate::handler::query as QueryHandler;
use crate::state::config;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub manager: HumanAddr,
    pub refund_denom: String,
    pub base_price: Decimal256,
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    config::store(
        &mut deps.storage,
        &config::Config {
            manager: msg.manager,
            refund_denom: msg.refund_denom,
            base_price: msg.base_price,
        },
    )?;

    Ok(InitResponse::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Configure {
        manager: Option<HumanAddr>,
        refund_denom: Option<String>,
        base_price: Option<Decimal256>,
    },
    Refund {
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
    },
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Configure {
            manager,
            refund_denom,
            base_price,
        } => ExecHandler::configure(deps, env, manager, refund_denom, base_price),
        HandleMsg::Refund { start_after, limit } => ExecHandler::refund(
            deps,
            env,
            match start_after {
                Some(start_after) => {
                    Option::from(deps.api.canonical_address(&start_after).unwrap())
                }
                None => None,
            },
            limit,
        ),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Buyers {
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
    },
    Simulate {
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
    },
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::Buyers { start_after, limit } => QueryHandler::buyers(
            deps,
            match start_after {
                Some(start_after) => {
                    Option::from(deps.api.canonical_address(&start_after).unwrap())
                }
                None => None,
            },
            limit,
        ),
        QueryMsg::Simulate { start_after, limit } => QueryHandler::simulate(
            deps,
            match start_after {
                Some(start_after) => {
                    Option::from(deps.api.canonical_address(&start_after).unwrap())
                }
                None => None,
            },
            limit,
        ),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _: &mut Extern<S, A, Q>,
    _: Env,
    _: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
