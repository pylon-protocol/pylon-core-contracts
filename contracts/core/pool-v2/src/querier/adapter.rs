use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::*;
use pylon_core::adapter_msg::QueryMsg as AdapterQueryMsg;
use pylon_core::adapter_resp;

pub fn config(deps: Deps, adapter: String) -> StdResult<adapter_resp::ConfigResponse> {
    deps.querier
        .query::<adapter_resp::ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: adapter,
            msg: to_binary(&AdapterQueryMsg::Config {})?,
        }))
}

pub fn exchange_rate(deps: Deps, adapter: String, input_denom: String) -> StdResult<Decimal256> {
    let resp = deps
        .querier
        .query::<adapter_resp::ExchangeRateResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: adapter,
            msg: to_binary(&AdapterQueryMsg::ExchangeRate { input_denom })?,
        }))?;
    Ok(resp.exchange_rate)
}

pub fn deposit(deps: Deps, adapter: String, amount: Uint256) -> StdResult<Vec<CosmosMsg>> {
    deps.querier
        .query::<Vec<CosmosMsg>>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: adapter,
            msg: to_binary(&AdapterQueryMsg::Deposit { amount })?,
        }))
}

pub fn redeem(deps: Deps, adapter: String, amount: Uint256) -> StdResult<Vec<CosmosMsg>> {
    deps.querier
        .query::<Vec<CosmosMsg>>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: adapter,
            msg: to_binary(&AdapterQueryMsg::Redeem { amount })?,
        }))
}
