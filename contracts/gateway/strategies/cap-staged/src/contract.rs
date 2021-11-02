#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use pylon_gateway::cap_strategy_msg::{MigrateMsg, QueryMsg};
use pylon_gateway::cap_strategy_resp as resp;
use pylon_token::gov_msg::QueryMsg as GovQueryMsg;
use pylon_token::gov_resp::StakerResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

use crate::state;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub gov: String,
    pub stages: Vec<state::Stage>,
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
            gov: msg.gov,
            stages: msg.stages,
        })
        .unwrap();

    Ok(Response::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Configure {
        owner: Option<String>,
        gov: Option<String>,
        stages: Option<Vec<state::Stage>>,
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
        ExecuteMsg::Configure { owner, gov, stages } => {
            let mut config = state::config_r(deps.storage).load().unwrap();
            if config.owner != info.sender {
                return Err(StdError::generic_err(format!(
                    "expected: {}, actual: {}",
                    config.owner, info.sender,
                )));
            }

            if let Some(v) = owner {
                config.owner = v;
            }
            if let Some(v) = gov {
                config.gov = v;
            }
            if let Some(v) = stages {
                config.stages = v;
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
        QueryMsg::AvailableCapOf { address, amount } => {
            let config = state::config_r(deps.storage).load().unwrap();
            let staked: StakerResponse = deps
                .querier
                .query_wasm_smart(config.gov, &GovQueryMsg::Staker { address })?;

            let mut max_cap = Uint256::zero();
            for stage in config.stages.iter() {
                if stage.from <= Uint256::from(staked.balance) {
                    if let Some(to) = stage.to {
                        if Uint256::from(staked.balance) < to {
                            max_cap = max(max_cap, stage.max_cap);
                        }
                    } else {
                        max_cap = max(max_cap, stage.max_cap);
                    }
                }
            }
            to_binary(&resp::AvailableCapOfResponse {
                amount: Option::Some(max_cap - min(max_cap, amount)),
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
