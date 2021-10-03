use cosmwasm_std::*;
use pylon_core::factory_msg::QueryMsg as FactoryQueryMsg;
use pylon_core::factory_resp::ConfigResponse;

pub fn config(deps: Deps, factory: String) -> StdResult<ConfigResponse> {
    deps.querier
        .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: factory,
            msg: to_binary(&FactoryQueryMsg::Config {})?,
        }))
}
