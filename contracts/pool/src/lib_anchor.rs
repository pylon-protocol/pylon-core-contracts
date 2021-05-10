use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Coin, CosmosMsg, Extern, HumanAddr, Querier, QueryRequest,
    StdResult, Storage, Uint128, WasmMsg, WasmQuery,
};

use cw20::Cw20HandleMsg::Send as TokenSendMsg;
use moneymarket::market::Cw20HookMsg::RedeemStable as MarketRedeemStable;
use moneymarket::market::HandleMsg::DepositStable as MarketDepositStable;
use moneymarket::market::QueryMsg::{Config as MarketConfig, EpochState as MarketEpochState};
use moneymarket::market::{
    ConfigResponse as MarketConfigResponse, Cw20HookMsg,
    EpochStateResponse as MarketEpochStateResponse,
};
use moneymarket::querier::deduct_tax;

pub fn market_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
) -> StdResult<MarketConfigResponse> {
    let market_config: MarketConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(market)?,
            msg: to_binary(&MarketConfig {})?,
        }))?;

    Ok(market_config)
}

pub fn market_epoch_state<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
) -> StdResult<MarketEpochStateResponse> {
    let epoch_state: MarketEpochStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(market)?,
            msg: to_binary(&MarketEpochState { block_height: None })?,
        }))?;

    Ok(epoch_state)
}

pub fn market_deposit_stable_msg<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
    denom: &String,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(market)?,
        msg: to_binary(&MarketDepositStable {})?,
        send: vec![deduct_tax(
            deps,
            Coin {
                denom: denom.clone(),
                amount,
            },
        )?],
    })])
}

pub fn market_redeem_stable_msg<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
    token: &CanonicalAddr,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(token)?,
        msg: to_binary(&TokenSendMsg {
            contract: deps.api.human_address(market)?,
            amount,
            msg: Option::from(to_binary(&MarketRedeemStable {})?),
        })?,
        send: vec![],
    })])
}
