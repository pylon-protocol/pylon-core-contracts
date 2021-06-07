use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Coin, CosmosMsg, Extern, Querier, QueryRequest, StdResult,
    Storage, Uint128, WasmMsg, WasmQuery,
};

use cw20::Cw20HandleMsg;
use moneymarket::market::{ConfigResponse, Cw20HookMsg, EpochStateResponse, HandleMsg, QueryMsg};
use moneymarket::querier::deduct_tax;

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
) -> StdResult<ConfigResponse> {
    let market_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(market)?,
            msg: to_binary(&QueryMsg::Config {})?,
        }))?;

    Ok(market_config)
}

pub fn epoch_state<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
) -> StdResult<EpochStateResponse> {
    let epoch_state: EpochStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(market)?,
            msg: to_binary(&QueryMsg::EpochState { block_height: None })?,
        }))?;

    Ok(epoch_state)
}

pub fn deposit_stable_msg<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
    denom: &str,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(market)?,
        msg: to_binary(&HandleMsg::DepositStable {})?,
        send: vec![deduct_tax(
            deps,
            Coin {
                denom: denom.to_string(),
                amount,
            },
        )?],
    })])
}

pub fn redeem_stable_msg<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
    token: &CanonicalAddr,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(token)?,
        msg: to_binary(&Cw20HandleMsg::Send {
            contract: deps.api.human_address(market)?,
            amount,
            msg: Option::from(to_binary(&Cw20HookMsg::RedeemStable {})?),
        })?,
        send: vec![],
    })])
}
