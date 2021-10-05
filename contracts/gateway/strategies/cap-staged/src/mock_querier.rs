use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, Binary, Coin, ContractResult, OwnedDeps, Querier, QuerierResult, QueryRequest,
    StdResult, SystemError, SystemResult, WasmQuery,
};
use std::collections::HashMap;
use terra_cosmwasm::TerraQueryWrapper;

#[allow(dead_code)]
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, CustomMockWasmQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: CustomMockWasmQuerier {
            base: MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]),
            wasm_smart_query_handlers: HashMap::new(),
            wasm_raw_query_handlers: HashMap::new(),
        },
    }
}

pub type WasmQueryHandler = dyn Fn(&Binary) -> StdResult<Binary>;

pub struct CustomMockWasmQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    wasm_smart_query_handlers: HashMap<String, Box<WasmQueryHandler>>,
    wasm_raw_query_handlers: HashMap<String, Box<WasmQueryHandler>>,
}

impl Querier for CustomMockWasmQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {:?}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl CustomMockWasmQuerier {
    #[allow(dead_code)]
    pub fn register_wasm_smart_query_handler(
        &mut self,
        address: String,
        handler: Box<WasmQueryHandler>,
    ) {
        self.wasm_smart_query_handlers.insert(address, handler);
    }

    #[allow(dead_code)]
    pub fn register_wasm_raw_query_handler(
        &mut self,
        address: String,
        handler: Box<WasmQueryHandler>,
    ) {
        self.wasm_raw_query_handlers.insert(address, handler);
    }

    fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match request {
            QueryRequest::Wasm(wasm_request) => match wasm_request {
                WasmQuery::Smart { contract_addr, msg } => SystemResult::Ok(ContractResult::Ok(
                    self.wasm_smart_query_handlers
                        .get(contract_addr.as_str())
                        .expect("wasm: smart query handler not found")(msg)
                    .unwrap(),
                )),
                WasmQuery::Raw { contract_addr, key } => SystemResult::Ok(ContractResult::Ok(
                    self.wasm_raw_query_handlers
                        .get(contract_addr.as_str())
                        .expect("wasm: raw query handler not found")(key)
                    .unwrap(),
                )),
                _ => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: stringify!(request).to_string(),
                }),
            },
            _ => self.base.handle_query(request),
        }
    }
}
