use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{to_binary, ContractResult, QuerierResult, SystemError, SystemResult};
use std::collections::HashMap;

use crate::pool_v2_msg::QueryMsg;
use crate::pool_v2_resp;
use crate::test_constant::*;

#[derive(Clone)]
pub struct MockPool {
    pub configs: HashMap<String, pool_v2_resp::ConfigResponse>,
    pub rewards: HashMap<String, pool_v2_resp::ClaimableRewardResponse>,
}

impl Default for MockPool {
    fn default() -> Self {
        MockPool::new(
            &[(
                &TEST_POOL.to_string(),
                pool_v2_resp::ConfigResponse {
                    id: 0,
                    name: TEST_POOL.to_string(),
                    factory: TEST_CONTRACT.to_string(),
                    beneficiary: TEST_BENEFICIARY.to_string(),
                    yield_adapter: TEST_ADAPTER.to_string(),
                    input_denom: TEST_INPUT_DENOM.to_string(),
                    yield_token: TEST_TOKEN_YIELD.to_string(),
                    dp_token: TEST_TOKEN_POOL.to_string(),
                },
            )],
            &[(
                &TEST_POOL.to_string(),
                pool_v2_resp::ClaimableRewardResponse {
                    amount: Uint256::from(TEST_POOL_REWARD_AMOUNT),
                    fee: Uint256::from(TEST_POOL_REWARD_FEE),
                },
            )],
        )
    }
}

impl MockPool {
    pub fn handle_query(&self, pool: &str, msg: QueryMsg) -> QuerierResult {
        match msg {
            QueryMsg::Config {} => SystemResult::Ok(ContractResult::Ok(
                to_binary(self.configs.get(pool).unwrap()).unwrap(),
            )),
            QueryMsg::ClaimableReward {} => SystemResult::Ok(ContractResult::Ok(
                to_binary(self.rewards.get(pool).unwrap()).unwrap(),
            )),
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: stringify!(msg).to_string(),
            }),
        }
    }
}

impl MockPool {
    pub fn new(
        configs: &[(&String, pool_v2_resp::ConfigResponse)],
        rewards: &[(&String, pool_v2_resp::ClaimableRewardResponse)],
    ) -> Self {
        MockPool {
            configs: configs_to_map(configs),
            rewards: rewards_to_map(rewards),
        }
    }
}

pub fn configs_to_map(
    configs: &[(&String, pool_v2_resp::ConfigResponse)],
) -> HashMap<String, pool_v2_resp::ConfigResponse> {
    let mut config_map: HashMap<String, pool_v2_resp::ConfigResponse> = HashMap::new();
    for (address, config) in configs.iter() {
        config_map.insert(address.to_string(), config.clone());
    }

    config_map
}

pub fn rewards_to_map(
    rewards: &[(&String, pool_v2_resp::ClaimableRewardResponse)],
) -> HashMap<String, pool_v2_resp::ClaimableRewardResponse> {
    let mut reward_map: HashMap<String, pool_v2_resp::ClaimableRewardResponse> = HashMap::new();
    for (address, reward) in rewards.iter() {
        reward_map.insert(address.to_string(), reward.clone());
    }

    reward_map
}
