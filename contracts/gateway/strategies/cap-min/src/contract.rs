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

use crate::state;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub gov: String,
    pub minimum_stake_amount: Uint256,
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
            minimum_stake_amount: msg.minimum_stake_amount,
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
        minimum_stake_amount: Option<Uint256>,
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
            gov,
            minimum_stake_amount,
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
            if let Some(v) = gov {
                config.gov = v;
            }
            if let Some(v) = minimum_stake_amount {
                config.minimum_stake_amount = v;
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
        QueryMsg::AvailableCapOf { address, .. } => {
            let config = state::config_r(deps.storage).load().unwrap();
            let staked: StakerResponse = deps
                .querier
                .query_wasm_smart(config.gov, &GovQueryMsg::Staker { address })?;

            if config.minimum_stake_amount <= Uint256::from(staked.balance) {
                to_binary(&resp::AvailableCapOfResponse {
                    amount: Option::None,
                    unlimited: true,
                })
            } else {
                to_binary(&resp::AvailableCapOfResponse {
                    amount: Option::Some(Uint256::from(0u64)),
                    unlimited: false,
                })
            }
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
