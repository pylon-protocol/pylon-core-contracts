#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use pylon_gateway::cap_strategy_msg::{MigrateMsg, QueryMsg};
use pylon_gateway::cap_strategy_resp as resp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state;
use std::cmp::min;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub min_user_cap: Uint256,
    pub max_user_cap: Uint256,
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    state::config_w(deps.storage)
        .save(&state::Config {
            owner: info.sender.to_string(),
            min_user_cap: msg.min_user_cap,
            max_user_cap: msg.max_user_cap,
        })
        .unwrap();

    Ok(Response::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Configure {
        owner: Option<String>,
        min_user_cap: Option<Uint256>,
        max_user_cap: Option<Uint256>,
    },
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Configure {
            owner,
            min_user_cap,
            max_user_cap,
        } => {
            let mut config = state::config_r(deps.storage).load().unwrap();
            if config.owner != info.sender {
                return Err(StdError::generic_err(format!(
                    "expected: {}, actual: {}",
                    config.owner, info.sender
                )));
            }

            if let Some(v) = owner {
                config.owner = v;
            }
            if let Some(v) = min_user_cap {
                config.min_user_cap = v;
            }
            if let Some(v) = max_user_cap {
                config.max_user_cap = v;
            }

            state::config_w(deps.storage).save(&config).unwrap();
            Ok(Response::default())
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AvailableCapOf { amount, .. } => {
            let config = state::config_r(deps.storage).load().unwrap();
            to_binary(&resp::AvailableCapOfResponse {
                amount: Option::Some(config.max_user_cap - min(config.max_user_cap, amount)),
                unlimited: false,
            })
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
