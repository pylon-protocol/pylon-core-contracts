use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, CosmosMsg, Extern, HumanAddr, Querier, QueryRequest, StdResult,
    Storage, WasmMsg, WasmQuery,
};

use cosmwasm_bignumber::Decimal256;
use exchange_rate::msg::{HandleMsg, QueryMsg};
use exchange_rate::resp::ExchangeRateResponse;

pub fn update_msg<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    feeder: &CanonicalAddr,
    token: &CanonicalAddr,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(feeder)?,
        msg: to_binary(&HandleMsg::Update {
            token: deps.api.human_address(token)?,
        })?,
        send: vec![],
    })])
}

pub fn fetch<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    feeder: &CanonicalAddr,
    blocktime: Option<u64>,
    token: &HumanAddr,
) -> StdResult<Decimal256> {
    let resp: ExchangeRateResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps.api.human_address(feeder)?,
        msg: to_binary(&QueryMsg::ExchangeRateOf {
            token: token.clone(),
            blocktime,
        })?,
    }))?;

    Ok(resp.exchange_rate)
}
