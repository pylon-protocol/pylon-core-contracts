use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Extern, Querier, QueryRequest, StdResult, Storage, WasmQuery,
};

use pylon_core::adapter::{ExchangeRateResponse, QueryMsg as AdapterQueryMsg};

pub fn exchange_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    target: &CanonicalAddr,
    input_denom: &String,
) -> StdResult<Decimal256> {
    Ok(deps
        .querier
        .query::<ExchangeRateResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(target)?,
            msg: to_binary(&AdapterQueryMsg::ExchangeRate {
                input_denom: input_denom.clone(),
            })?,
        }))?
        .exchange_rate)
}
