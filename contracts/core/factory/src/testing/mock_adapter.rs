use crate::testing::constants::{TEST_INPUT_DENOM, TEST_YIELD_TOKEN};
use cosmwasm_std::{to_binary, HumanAddr, QuerierResult, SystemError};
use pylon_core::adapter_msg::QueryMsg as AdapterQueryMsg;
use pylon_core::adapter_resp;

#[derive(Clone)]
pub struct MockAdapter {
    pub input_denom: String,
    pub yield_token: HumanAddr,
}

impl Default for MockAdapter {
    fn default() -> Self {
        MockAdapter {
            input_denom: TEST_INPUT_DENOM.to_string(),
            yield_token: HumanAddr::from(TEST_YIELD_TOKEN),
        }
    }
}

impl MockAdapter {
    pub fn handle_query(&self, msg: AdapterQueryMsg) -> QuerierResult {
        match msg {
            AdapterQueryMsg::Config {} => Ok(to_binary(&adapter_resp::ConfigResponse {
                input_denom: self.input_denom.clone(),
                yield_token: self.yield_token.clone(),
            })),
            _ => Err(SystemError::UnsupportedRequest {
                kind: stringify!(msg).to_string(),
            }),
        }
    }
}
