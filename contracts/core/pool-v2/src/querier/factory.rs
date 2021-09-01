use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Extern, Querier, QueryRequest, StdResult, Storage, WasmQuery,
};
use pylon_core::factory_msg::QueryMsg as FactoryQueryMsg;
use pylon_core::factory_resp::ConfigResponse;

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    factory: &CanonicalAddr,
) -> StdResult<ConfigResponse> {
    deps.querier
        .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(factory)?,
            msg: to_binary(&FactoryQueryMsg::Config {})?,
        }))
}
