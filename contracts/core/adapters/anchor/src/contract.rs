use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse,
    MigrateResult, Querier, StdResult, Storage,
};
use pylon_core::adapter_msg::{HandleMsg, QueryMsg};
use pylon_core::adapter_resp;

use crate::anchor;
use crate::config;
use crate::msg::{InitMsg, MigrateMsg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let moneymarket = deps.api.canonical_address(&msg.moneymarket)?;
    let anchor_config = anchor::config(deps, &moneymarket)?;

    let config = config::Config {
        owner: deps.api.canonical_address(&env.message.sender)?,
        moneymarket,
        input_denom: anchor_config.stable_denom.clone(),
        yield_token: deps.api.canonical_address(&anchor_config.aterra_contract)?,
    };

    config::store(&mut deps.storage, &config)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: HandleMsg,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = config::read(&deps.storage)?;

            to_binary(&adapter_resp::ConfigResponse {
                input_denom: config.input_denom.clone(),
                yield_token: deps.api.human_address(&config.yield_token)?,
            })
        }
        QueryMsg::ExchangeRate { input_denom: _ } => {
            let config = config::read(&deps.storage)?;
            let epoch_state = anchor::epoch_state(&deps, &config.moneymarket)?;

            to_binary(&adapter_resp::ExchangeRateResponse {
                exchange_rate: epoch_state.exchange_rate,
                yield_token_supply: epoch_state.aterra_supply,
            })
        }
        QueryMsg::Deposit { amount } => {
            let config = config::read(&deps.storage)?;

            to_binary(&anchor::deposit_stable_msg(
                deps,
                &config.moneymarket,
                &config.input_denom,
                amount.into(),
            )?)
        }
        QueryMsg::Redeem { amount } => {
            let config = config::read(&deps.storage)?;

            to_binary(&anchor::redeem_stable_msg(
                deps,
                &config.moneymarket,
                &config.yield_token,
                amount.into(),
            )?)
        }
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
