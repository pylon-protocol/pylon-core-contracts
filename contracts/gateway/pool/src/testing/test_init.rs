use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::Response;
use pylon_gateway::time_range::TimeRange;
use std::ops::Add;

use crate::contract;
use crate::state::{config, reward};
use crate::testing::constants::TEST_OWNER;
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = utils::init_msg();
    let env = mock_env();
    let info = mock_info(TEST_OWNER, &[]);
    let res = contract::instantiate(deps.as_mut(), env, info.clone(), msg.clone()).unwrap();
    assert_eq!(res, Response::default());

    let config = config::read(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            owner: info.sender.to_string(),
            // share
            share_token: msg.share_token,
            deposit_config: config::DepositConfig {
                time: TimeRange {
                    start: msg.start,
                    finish: 0,
                    inverse: false,
                },
                user_cap: Uint256::zero(),
                total_cap: Uint256::zero(),
            },
            withdraw_time: vec![TimeRange {
                start: msg.start,
                finish: msg.start.add(msg.period),
                inverse: true,
            }],
            // reward
            reward_token: msg.reward_token,
            claim_time: TimeRange {
                start: msg.start.add(msg.cliff),
                finish: 0,
                inverse: false,
            },
            distribution_config: config::DistributionConfig {
                time: TimeRange {
                    start: msg.start,
                    finish: msg.start.add(msg.period),
                    inverse: false,
                },
                reward_rate: Decimal256::from_ratio(msg.reward_amount, Uint256::from(msg.period)),
            },
            cap_strategy: msg.cap_strategy,
        }
    );

    let reward = reward::read(&deps.storage).unwrap();
    assert_eq!(
        reward,
        reward::Reward {
            total_deposit: Uint256::zero(),
            last_update_time: msg.start,
            reward_per_token_stored: Decimal256::zero(),
        }
    );
}
