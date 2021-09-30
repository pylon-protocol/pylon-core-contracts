use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, MigrateResponse,
    Querier, StdError, StdResult, Storage,
};
use pylon_core::adapter_msg::{HandleMsg, QueryMsg};
use pylon_core::adapter_resp;

use crate::config;
use crate::market;
use crate::msg::{InstantiateMsg, MigrateMsg};

#[allow(dead_code)]
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InstantiateMsg,
) -> Result<InitResponse, StdError> {
    let market_config = market::config(deps, msg.moneymarket.clone())?;

    let config = config::Config {
        owner: env.message.sender.to_string(),
        moneymarket: msg.moneymarket,
        input_denom: market_config.input_denom,
        yield_token: market_config.output_token,
    };

    config::store(&mut deps.storage, &config)?;

    Ok(InitResponse::default())
}

#[allow(dead_code)]
pub fn handle<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: HandleMsg,
) -> Result<HandleResponse, StdError> {
    Ok(HandleResponse::default())
}

#[allow(dead_code)]
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = config::read(&deps.storage)?;

            to_binary(&adapter_resp::ConfigResponse {
                input_denom: config.input_denom.clone(),
                yield_token: HumanAddr::from(config.yield_token),
            })
        }
        QueryMsg::ExchangeRate { input_denom: _ } => {
            let config = config::read(&deps.storage)?;
            let market_config = market::config(&deps, config.moneymarket)?;

            to_binary(&adapter_resp::ExchangeRateResponse {
                exchange_rate: market_config.exchange_rate,
                yield_token_supply: Uint256::zero(),
            })
        }
        QueryMsg::Deposit { amount } => {
            let config = config::read(&deps.storage)?;

            to_binary(&market::deposit_stable_msg(
                &deps,
                config.moneymarket,
                &config.input_denom,
                amount.into(),
            )?)
        }
        QueryMsg::Redeem { amount } => {
            let config = config::read(&deps.storage)?;

            to_binary(&market::redeem_stable_msg(
                &deps,
                config.moneymarket,
                config.yield_token,
                amount.into(),
            )?)
        }
    }
}

#[allow(dead_code)]
pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<MigrateResponse, StdError> {
    Ok(MigrateResponse::default())
}
