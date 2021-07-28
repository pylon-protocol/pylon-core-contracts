use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Extern, Querier, QueryRequest, StdResult, Storage, WasmQuery,
};

use pylon_core::factory_msg::QueryMsg as FactoryQueryMsg;
use pylon_core::factory_resp::AdapterInfoResponse;

pub fn fee_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    factory: &CanonicalAddr,
    adapter: &CanonicalAddr,
) -> StdResult<Decimal256> {
    Ok(deps
        .querier
        .query::<AdapterInfoResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(factory)?,
            msg: to_binary(&FactoryQueryMsg::AdapterInfo {
                address: deps.api.human_address(adapter)?,
            })?,
        }))?
        .fee_rate)
}
