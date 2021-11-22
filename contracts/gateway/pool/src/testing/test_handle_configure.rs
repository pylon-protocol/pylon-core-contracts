use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_info};
use cosmwasm_std::Timestamp;
use pylon_gateway::pool_msg::{ConfigureMsg, ExecuteMsg};
use pylon_gateway::time_range::TimeRange;
use std::ops::Mul;
use std::str::FromStr;

use crate::contract;
use crate::state::config;
use crate::testing::constants::*;
use crate::testing::utils;

#[test]
fn handle_configure_deposit() {
    let mut deps = mock_dependencies(&[]);
    let (env, owner) = utils::initialize(&mut deps);
    let user = mock_info(TEST_USER, &[]);

    let start = TEST_POOL_START + TEST_POOL_PERIOD;
    let finish = start + TEST_POOL_PERIOD;
    let user_cap = Uint256::from(1000000000u64);
    let total_cap = Uint256::from(10000000000000000u64);

    let msg = ExecuteMsg::Configure(ConfigureMsg::Deposit {
        start: Option::from(start),
        finish: Option::from(finish),
        user_cap: Option::from(user_cap),
        total_cap: Option::from(total_cap),
    });
    let err = contract::execute(deps.as_mut(), env.clone(), user, msg.clone())
        .expect_err("testing: should fail if non-owner executes this msg");
    utils::assert_generic_err("cfg_deposit: check_owner", err);
    let res = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: handle configure::deposit");
    assert_eq!(res.data, None);
    assert_eq!(res.messages, vec![]);

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config.deposit_config,
        config::DepositConfig {
            time: TimeRange {
                start,
                finish,
                inverse: false
            },
            user_cap,
            total_cap
        }
    );
}

#[test]
fn handle_configure_withdraw() {
    let mut deps = mock_dependencies(&[]);
    let (env, owner) = utils::initialize(&mut deps);
    let user = mock_info(TEST_USER, &[]);

    let strategy = vec![(100, 200, false), (300, 400, false), (0, 500, true)];

    let msg = ExecuteMsg::Configure(ConfigureMsg::Withdraw {
        strategy: strategy.clone(),
    });
    let err = contract::execute(deps.as_mut(), env.clone(), user, msg.clone())
        .expect_err("testing: should fail if non-owner executes this msg");
    utils::assert_generic_err("cfg_withdraw: check_owner", err);
    let res = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: handle configure::withdraw");
    assert_eq!(res.data, None);
    assert_eq!(res.messages, vec![]);

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config.withdraw_time,
        strategy
            .iter()
            .map(|(start, finish, inverse)| TimeRange {
                start: *start,
                finish: *finish,
                inverse: *inverse
            })
            .collect::<Vec<TimeRange>>()
    );
}

#[test]
fn handle_configure_claim() {
    let mut deps = mock_dependencies(&[]);
    let (env, owner) = utils::initialize(&mut deps);
    let user = mock_info(TEST_USER, &[]);

    let start = 300;
    let finish = 600;

    let msg = ExecuteMsg::Configure(ConfigureMsg::Claim {
        start: Option::from(start),
        finish: Option::from(finish),
    });
    let err = contract::execute(deps.as_mut(), env.clone(), user, msg.clone())
        .expect_err("testing: should fail if non-owner executes this msg");
    utils::assert_generic_err("cfg_claim: check_owner", err);
    let res = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: handle configure::claim");
    assert_eq!(res.data, None);
    assert_eq!(res.messages, vec![]);

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config.claim_time,
        TimeRange {
            start,
            finish,
            inverse: false
        }
    );
}

#[test]
fn handle_configure_distribution_add_reward() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, owner) = utils::initialize(&mut deps);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START);
    let user = mock_info(TEST_USER, &[]);

    let amount = Uint256::from(500u64);
    let prev_config = config::read(&deps.storage).unwrap();

    let msg = ExecuteMsg::Configure(ConfigureMsg::AddReward { amount });
    let err = contract::execute(deps.as_mut(), env.clone(), user, msg.clone())
        .expect_err("testing: should fail if non-owner executes this msg");
    utils::assert_generic_err("cfg_distribution_ar: check_owner", err);
    let res = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: handle configure::distribution");
    assert_eq!(res.data, None);
    assert_eq!(res.messages, vec![]);

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config.distribution_config,
        config::DistributionConfig {
            time: prev_config.distribution_config.time,
            reward_rate: prev_config
                .distribution_config
                .reward_rate
                .mul(Decimal256::from_str("1.5").unwrap()),
        }
    );
}

#[test]
fn handle_configure_distribution_add_reward_middle_of_period() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, owner) = utils::initialize(&mut deps);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START + TEST_POOL_PERIOD / 2);
    let user = mock_info(TEST_USER, &[]);

    let amount = Uint256::from(500u64);
    let prev_config = config::read(&deps.storage).unwrap();

    let msg = ExecuteMsg::Configure(ConfigureMsg::AddReward { amount });
    let err = contract::execute(deps.as_mut(), env.clone(), user, msg.clone())
        .expect_err("testing: should fail if non-owner executes this msg");
    utils::assert_generic_err("cfg_distribution_ar: check_owner", err);
    let res = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: handle configure::distribution");
    assert_eq!(res.data, None);
    assert_eq!(res.messages, vec![]);

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config.distribution_config,
        config::DistributionConfig {
            time: prev_config.distribution_config.time,
            reward_rate: prev_config
                .distribution_config
                .reward_rate
                .mul(Decimal256::from_str("2.0").unwrap()),
        }
    );
}

#[test]
fn handle_configure_distribution_sub_reward() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, owner) = utils::initialize(&mut deps);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START);
    let user = mock_info(TEST_USER, &[]);

    let amount = Uint256::from(500u64);
    let prev_config = config::read(&deps.storage).unwrap();

    let msg = ExecuteMsg::Configure(ConfigureMsg::SubReward { amount });
    let err = contract::execute(deps.as_mut(), env.clone(), user, msg.clone())
        .expect_err("testing: should fail if non-owner executes this msg");
    utils::assert_generic_err("cfg_distribution_sr: check_owner", err);
    let res = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: handle configure::distribution");
    assert_eq!(res.data, None);
    assert_eq!(res.messages, vec![]);

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config.distribution_config,
        config::DistributionConfig {
            time: prev_config.distribution_config.time,
            reward_rate: prev_config
                .distribution_config
                .reward_rate
                .mul(Decimal256::from_str("0.5").unwrap()),
        }
    );
}

#[test]
fn handle_configure_distribution_sub_reward_middle_of_period() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, owner) = utils::initialize(&mut deps);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START + TEST_POOL_PERIOD / 2);
    let user = mock_info(TEST_USER, &[]);

    let amount = Uint256::from(250u64);
    let prev_config = config::read(&deps.storage).unwrap();

    let msg = ExecuteMsg::Configure(ConfigureMsg::SubReward { amount });
    let err = contract::execute(deps.as_mut(), env.clone(), user, msg.clone())
        .expect_err("testing: should fail if non-owner executes this msg");
    utils::assert_generic_err("cfg_distribution_sr: check_owner", err);
    let res = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: handle configure::distribution");
    assert_eq!(res.data, None);
    assert_eq!(res.messages, vec![]);

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config.distribution_config,
        config::DistributionConfig {
            time: prev_config.distribution_config.time,
            reward_rate: prev_config
                .distribution_config
                .reward_rate
                .mul(Decimal256::from_str("0.5").unwrap()),
        }
    );
}
