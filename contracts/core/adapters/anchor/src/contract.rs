#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use pylon_core::adapter_msg::{HandleMsg, QueryMsg};
use pylon_core::adapter_resp;

use crate::anchor;
use crate::config;
use crate::msg::{InstantiateMsg, MigrateMsg};

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    let anchor_config = anchor::config(&deps.as_ref(), msg.moneymarket.clone())?;

    let config = config::Config {
        owner: info.sender.to_string(),
        moneymarket: msg.moneymarket,
        input_denom: anchor_config.stable_denom.clone(),
        yield_token: anchor_config.aterra_contract,
    };

    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: HandleMsg,
) -> Result<Response, StdError> {
    Ok(Response::default())
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = config::read(deps.storage)?;

            to_binary(&adapter_resp::ConfigResponse {
                input_denom: config.input_denom.clone(),
                yield_token: config.yield_token,
            })
        }
        QueryMsg::ExchangeRate { input_denom: _ } => {
            let config = config::read(deps.storage)?;
            let epoch_state = anchor::epoch_state(deps, config.moneymarket)?;

            to_binary(&adapter_resp::ExchangeRateResponse {
                exchange_rate: epoch_state.exchange_rate,
                yield_token_supply: epoch_state.aterra_supply,
            })
        }
        QueryMsg::Deposit { amount } => {
            let config = config::read(deps.storage)?;

            to_binary(&anchor::deposit_stable_msg(
                deps,
                config.moneymarket,
                &config.input_denom,
                amount.into(),
            )?)
        }
        QueryMsg::Redeem { amount } => {
            let config = config::read(deps.storage)?;

            to_binary(&anchor::redeem_stable_msg(
                deps,
                config.moneymarket,
                config.yield_token,
                amount.into(),
            )?)
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, StdError> {
    Ok(Response::default())
}
