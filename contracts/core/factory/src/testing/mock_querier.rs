use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, Api, Coin, Extern, HumanAddr, Querier, QuerierResult, QueryRequest,
    SystemError, WasmQuery,
};
use terra_cosmwasm::TerraQueryWrapper;

use crate::testing::constants::{TEST_POOL, TEST_YIELD_ADAPTER};
use crate::testing::mock_adapter::MockAdapter;
use crate::testing::mock_pool::MockPool;
use crate::testing::mock_token::MockToken;

pub fn mock_dependencies(
    canonical_length: usize,
    contract_balance: &[Coin],
) -> Extern<MockStorage, MockApi, CustomMockQuerier> {
    let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
    let api = MockApi::new(canonical_length);

    Extern {
        storage: MockStorage::default(),
        api: api.clone(),
        querier: CustomMockQuerier::new(
            MockQuerier::new(&[(&contract_addr, contract_balance)]),
            canonical_length,
            api,
        ),
    }
}

pub struct CustomMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    pool: MockPool,
    token: MockToken,
    adapter: MockAdapter,
    canonical_length: usize,
}

impl Querier for CustomMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<TerraQueryWrapper> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {:?}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl CustomMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg: bin_msg,
            }) => {
                if contract_addr.to_string().starts_with("token_") {
                    self.token
                        .handle_query(contract_addr, from_binary(bin_msg).unwrap())
                } else if contract_addr.to_string().starts_with("yield_adapter") {
                    self.adapter.handle_query(from_binary(bin_msg).unwrap())
                } else {
                    match contract_addr.to_string().as_str() {
                        TEST_POOL => self.pool.handle_query(
                            &HumanAddr::from(TEST_POOL.to_string()),
                            from_binary(bin_msg).unwrap(),
                        ),
                        _ => Err(SystemError::UnsupportedRequest {
                            kind: contract_addr.to_string(),
                        }),
                    }
                }
            }
            _ => self.base.handle_query(request),
        }
    }
}

impl CustomMockQuerier {
    pub fn new<A: Api>(
        base: MockQuerier<TerraQueryWrapper>,
        canonical_length: usize,
        _api: A,
    ) -> Self {
        CustomMockQuerier {
            base,
            pool: MockPool::default(),
            token: MockToken::default(),
            adapter: MockAdapter::default(),
            canonical_length,
        }
    }

    pub fn with_pool(&mut self, pool: MockPool) {
        self.pool = pool;
    }

    pub fn with_token(&mut self, token: MockToken) {
        self.token = token;
    }

    pub fn with_adapter(&mut self, adapter: MockAdapter) {
        self.adapter = adapter;
    }
}
