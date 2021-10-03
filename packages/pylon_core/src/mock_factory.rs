use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{to_binary, ContractResult, QuerierResult, SystemError, SystemResult};

use crate::factory_msg::QueryMsg;
use crate::factory_resp::ConfigResponse;
use crate::test_constant::*;

#[derive(Clone)]
pub struct MockFactory {
    pub owner: String,
    pub pool_code_id: u64,
    pub token_code_id: u64,
    pub fee_rate: Decimal256,
    pub fee_collector: String,
}

impl Default for MockFactory {
    fn default() -> Self {
        MockFactory {
            owner: TEST_FACTORY_OWNER.to_string(),
            pool_code_id: TEST_POOL_ID,
            token_code_id: TEST_TOKEN_CODE_ID,
            fee_rate: Decimal256::percent(TEST_FACTORY_FEE_RATE),
            fee_collector: TEST_FACTORY_FEE_COLLECTOR.to_string(),
        }
    }
}

impl MockFactory {
    pub fn handle_query(&self, msg: QueryMsg) -> QuerierResult {
        match msg {
            QueryMsg::Config {} => SystemResult::Ok(ContractResult::Ok(
                to_binary(&ConfigResponse {
                    owner: self.owner.clone(),
                    pool_code_id: self.pool_code_id,
                    token_code_id: self.token_code_id,
                    fee_rate: self.fee_rate,
                    fee_collector: self.fee_collector.clone(),
                })
                .unwrap(),
            )),
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: stringify!(msg).to_string(),
            }),
        }
    }
}
