use crate::testing::constants::{
    TEST_BENEFICIARY, TEST_INPUT_DENOM, TEST_POOL, TEST_POOL_REWARD_AMOUNT, TEST_POOL_REWARD_FEE,
    TEST_POOL_TOKEN, TEST_YIELD_ADAPTER, TEST_YIELD_TOKEN,
};
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::{to_binary, HumanAddr, QuerierResult, SystemError};
use pylon_core::pool_v2_msg::QueryMsg as PoolQueryMsg;
use pylon_core::pool_v2_resp as pool_resp;
use std::collections::HashMap;

#[derive(Clone)]
pub struct MockPool {
    pub configs: HashMap<HumanAddr, pool_resp::ConfigResponse>,
    pub rewards: HashMap<HumanAddr, pool_resp::ClaimableRewardResponse>,
}

impl Default for MockPool {
    fn default() -> Self {
        MockPool::new(
            &[(
                &TEST_POOL.to_string(),
                pool_resp::ConfigResponse {
                    id: 0,
                    name: TEST_POOL.to_string(),
                    factory: HumanAddr::from(MOCK_CONTRACT_ADDR),
                    beneficiary: HumanAddr::from(TEST_BENEFICIARY),
                    yield_adapter: HumanAddr::from(TEST_YIELD_ADAPTER),
                    input_denom: TEST_INPUT_DENOM.to_string(),
                    yield_token: HumanAddr::from(TEST_YIELD_TOKEN),
                    dp_token: HumanAddr::from(TEST_POOL_TOKEN),
                },
            )],
            &[(
                &TEST_POOL.to_string(),
                pool_resp::ClaimableRewardResponse {
                    amount: Uint256::from(TEST_POOL_REWARD_AMOUNT),
                    fee: Uint256::from(TEST_POOL_REWARD_FEE),
                },
            )],
        )
    }
}

impl MockPool {
    pub fn handle_query(&self, pool: &HumanAddr, msg: PoolQueryMsg) -> QuerierResult {
        match msg {
            PoolQueryMsg::Config {} => Ok(to_binary(self.configs.get(pool).unwrap())),
            PoolQueryMsg::ClaimableReward {} => Ok(to_binary(self.rewards.get(pool).unwrap())),
            _ => Err(SystemError::UnsupportedRequest {
                kind: stringify!(msg).to_string(),
            }),
        }
    }
}

impl MockPool {
    pub fn new(
        configs: &[(&String, pool_resp::ConfigResponse)],
        rewards: &[(&String, pool_resp::ClaimableRewardResponse)],
    ) -> Self {
        MockPool {
            configs: configs_to_map(configs),
            rewards: rewards_to_map(rewards),
        }
    }
}

pub fn configs_to_map(
    configs: &[(&String, pool_resp::ConfigResponse)],
) -> HashMap<HumanAddr, pool_resp::ConfigResponse> {
    let mut config_map: HashMap<HumanAddr, pool_resp::ConfigResponse> = HashMap::new();
    for (address, config) in configs.iter() {
        config_map.insert(HumanAddr::from(address.to_string()), config.clone());
    }

    config_map
}

pub fn rewards_to_map(
    rewards: &[(&String, pool_resp::ClaimableRewardResponse)],
) -> HashMap<HumanAddr, pool_resp::ClaimableRewardResponse> {
    let mut reward_map: HashMap<HumanAddr, pool_resp::ClaimableRewardResponse> = HashMap::new();
    for (address, reward) in rewards.iter() {
        reward_map.insert(HumanAddr::from(address.to_string()), reward.clone());
    }

    reward_map
}
