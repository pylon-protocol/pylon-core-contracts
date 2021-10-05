use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};

use cosmwasm_std::{
    from_binary, from_slice, to_binary, Api, Coin, ContractResult, OwnedDeps, Querier,
    QuerierResult, QueryRequest, SystemError, SystemResult, WasmQuery,
};
use pylon_utils::mock_tax::MockTax;
use pylon_utils::mock_token::MockToken;
use terra_cosmwasm::{TaxCapResponse, TaxRateResponse, TerraQuery, TerraQueryWrapper, TerraRoute};

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
    tax: MockTax,
    token: MockToken,
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
            QueryRequest::Custom(TerraQueryWrapper { route, query_data }) => {
                if &TerraRoute::Treasury == route {
                    match query_data {
                        TerraQuery::TaxRate {} => {
                            let res = TaxRateResponse {
                                rate: self.tax.rate,
                            };
                            SystemResult::Ok(ContractResult::Ok(to_binary(&res).unwrap()))
                        }
                        TerraQuery::TaxCap { denom } => {
                            let cap = self.tax.caps.get(denom).copied().unwrap_or_default();
                            let res = TaxCapResponse { cap };
                            SystemResult::Ok(ContractResult::Ok(to_binary(&res).unwrap()))
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
            token: MockToken::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_tax(&mut self, tax: MockTax) {
        self.tax = tax;
    }

    #[allow(dead_code)]
    pub fn with_token(&mut self, token: MockToken) {
        self.token = token;
    }
}
