use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::InitResponse;
use std::ops::{Add, Mul};

use crate::contract;
use crate::state::{config, reward, time_range};
use crate::testing::constants::TEST_OWNER;
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = utils::init_msg();
    let env = mock_env(TEST_OWNER, &[]);
    let res = contract::init(&mut deps, env.clone(), msg.clone()).unwrap();
    assert_eq!(res, InitResponse::default());

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            owner: env.message.sender,
            // share
            share_token: msg.share_token.clone(),
            deposit_config: config::DepositConfig {
                time: time_range::TimeRange {
                    start: msg.start,
                    finish: msg.start.add(msg.period),
                    inverse: false,
                },
                user_cap: Uint256::zero(),
                total_cap: Uint256::zero(),
            },
            withdraw_time: vec![time_range::TimeRange {
                start: msg.start,
                finish: msg.start.add(msg.period),
                inverse: true,
            }],
            // reward
            reward_token: msg.reward_token,
            claim_time: time_range::TimeRange {
                start: msg.cliff,
                finish: msg.start.add(msg.period),
                inverse: false,
            },
            distribution_config: config::DistributionConfig {
                time: time_range::TimeRange {
                    start: msg.start,
                    finish: msg.start.add(msg.period),
                    inverse: false,
                },
                reward_rate: msg.reward_rate,
                total_reward_amount: Uint256::from(msg.period).mul(msg.reward_rate),
            },
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
