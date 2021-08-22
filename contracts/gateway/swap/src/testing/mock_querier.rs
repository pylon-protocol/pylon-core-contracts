use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Api, Coin, Empty, Extern, HumanAddr, Querier,
    QuerierResult, QueryRequest, SystemError, Uint128, WasmMsg, WasmQuery,
};
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

use crate::testing::constants::TEST_STAKING;
use crate::testing::mock_staking::MockStaking;
use crate::testing::mock_tax::MockTax;
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
    tax: MockTax,
    token: MockToken,
    staking: MockStaking,
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
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if &TerraRoute::Treasury == route {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax.rate,
                            };
                            Ok(to_binary(&res))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self.tax.caps.get(denom).copied().unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            Ok(to_binary(&res))
                        }
                        _ => panic!("DO NOT ENTER HERE"),
                    }
                } else {
                    panic!("DO NOT ENTER HERE")
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg: bin_msg,
            }) => {
                if contract_addr.to_string().starts_with("token_") {
                    self.token
                        .handle_query(contract_addr, from_binary(bin_msg).unwrap())
                } else {
                    match contract_addr.to_string().as_str() {
                        TEST_STAKING => self.staking.handle_query(from_binary(bin_msg).unwrap()),
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
            tax: MockTax::default(),
            token: MockToken::default(),
            staking: MockStaking::default(),
            canonical_length,
        }
    }

    pub fn with_tax(&mut self, tax: MockTax) {
        self.tax = tax;
    }

    pub fn with_token(&mut self, token: MockToken) {
        self.token = token;
    }

    pub fn with_staking(&mut self, staking: MockStaking) {
        self.staking = staking;
    }
}
