use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::*;
use cw20::TokenInfoResponse;
use pylon_core::mock_adapter::MockAdapter;
use pylon_core::mock_pool::MockPool;
use pylon_core::test_constant::*;
use pylon_utils::mock_token::MockToken;
use terra_cosmwasm::TerraQueryWrapper;

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, CustomMockQuerier> {
    let api = MockApi::default();

    OwnedDeps {
        storage: MockStorage::default(),
        api,
        querier: CustomMockQuerier::new(
            MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]),
            api,
        ),
    }
}

pub struct CustomMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    pub pool: MockPool,
    pub token: MockToken,
    pub adapter: MockAdapter,
}

impl Querier for CustomMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
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

impl CustomMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<TerraQueryWrapper>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg: bin_msg,
            }) => {
                if contract_addr.starts_with("token_") {
                    self.token
                        .handle_query(contract_addr, from_binary(bin_msg).unwrap())
                } else if contract_addr.starts_with("adapter") {
                    self.adapter.handle_query(from_binary(bin_msg).unwrap())
                } else {
                    match contract_addr.as_str() {
                        TEST_POOL => self
                            .pool
                            .handle_query(&TEST_POOL.to_string(), from_binary(bin_msg).unwrap()),
                        _ => SystemResult::Err(SystemError::UnsupportedRequest {
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
    pub fn new<A: Api>(base: MockQuerier<TerraQueryWrapper>, _api: A) -> Self {
        CustomMockQuerier {
            base,
            pool: MockPool::default(),
            token: MockToken::new(
                &[
                    (
                        &TEST_TOKEN_YIELD.to_string(),
                        TokenInfoResponse {
                            name: TEST_TOKEN_YIELD.to_string(),
                            symbol: "TNT".to_string(),
                            decimals: 6u8,
                            total_supply: Uint128::from(TEST_TOKEN_YIELD_SUPPLY),
                        },
                    ),
                    (
                        &TEST_TOKEN_POOL.to_string(),
                        TokenInfoResponse {
                            name: TEST_TOKEN_POOL.to_string(),
                            symbol: "TNT".to_string(),
                            decimals: 6u8,
                            total_supply: Uint128::from(TEST_TOKEN_POOL_SUPPLY),
                        },
                    ),
                ],
                &[],
            ),
            adapter: MockAdapter::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_pool(&mut self, pool: MockPool) {
        self.pool = pool;
    }

    #[allow(dead_code)]
    pub fn with_token(&mut self, token: MockToken) {
        self.token = token;
    }

    #[allow(dead_code)]
    pub fn with_adapter(&mut self, adapter: MockAdapter) {
        self.adapter = adapter;
    }
}
