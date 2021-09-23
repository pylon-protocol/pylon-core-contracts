use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{to_binary, HumanAddr, QuerierResult, SystemError};
use pylon_core::factory_msg::QueryMsg;
use pylon_core::factory_resp::ConfigResponse;

use crate::testing::constants::{
    TEST_FACTORY_FEE_COLLECTOR, TEST_FACTORY_FEE_RATE, TEST_FACTORY_OWNER, TEST_POOL_ID,
    TEST_TOKEN_CODE_ID,
};

#[derive(Clone)]
pub struct MockFactory {
    pub owner: HumanAddr,
    pub pool_code_id: u64,
    pub token_code_id: u64,
    pub fee_rate: Decimal256,
    pub fee_collector: HumanAddr,
}

impl Default for MockFactory {
    fn default() -> Self {
        MockFactory {
            owner: HumanAddr::from(TEST_FACTORY_OWNER),
            pool_code_id: TEST_POOL_ID,
            token_code_id: TEST_TOKEN_CODE_ID,
            fee_rate: Decimal256::percent(TEST_FACTORY_FEE_RATE),
            fee_collector: HumanAddr::from(TEST_FACTORY_FEE_COLLECTOR),
        }
    }
}

impl MockFactory {
    pub fn handle_query(&self, msg: QueryMsg) -> QuerierResult {
        match msg {
            QueryMsg::Config {} => Ok(to_binary(&ConfigResponse {
                owner: self.owner.clone(),
                pool_code_id: self.pool_code_id,
                token_code_id: self.token_code_id,
                fee_rate: self.fee_rate,
                fee_collector: self.fee_collector.clone(),
            })),
            _ => Err(SystemError::UnsupportedRequest {
                kind: stringify!(msg).to_string(),
            }),
        }
    }
}
