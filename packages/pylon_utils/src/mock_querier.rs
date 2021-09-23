use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_slice, to_binary, Api, Coin, Extern, HumanAddr, Querier, QuerierResult, QueryRequest,
    SystemError,
};
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

use crate::mock_tax::MockTax;

pub fn mock_dependencies(
    canonical_length: usize,
    contract_balance: &[Coin],
) -> Extern<MockStorage, MockApi, CustomMockQuerier> {
    let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
    let api = MockApi::new(canonical_length);

    Extern {
        storage: MockStorage::default(),
        api,
        querier: CustomMockQuerier::new(
            MockQuerier::new(&[(&contract_addr, contract_balance)]),
            api,
        ),
    }
}

pub struct CustomMockQuerier {
    base: MockQuerier<TerraQueryWrapper>,
    tax: MockTax,
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
            _ => self.base.handle_query(request),
        }
    }
}

impl CustomMockQuerier {
    pub fn new<A: Api>(base: MockQuerier<TerraQueryWrapper>, _api: A) -> Self {
        CustomMockQuerier {
            base,
            tax: MockTax::default(),
        }
    }

    pub fn with_tax(&mut self, tax: MockTax) {
        self.tax = tax;
    }
}
