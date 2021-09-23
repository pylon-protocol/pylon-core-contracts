use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Api, HumanAddr};
use pylon_gateway::pool_msg::HandleMsg;
use std::ops::{Add, Div, Mul};

use crate::contract;
use crate::state::{config, reward, user};
use crate::testing::constants::*;
use crate::testing::utils;

const DEPOSIT_AMOUNT: u64 = 1000000u64;

#[test]
fn handle_update_without_target() {
    let mut deps = mock_dependencies(20, &[]);
    let mut env = utils::initialize(&mut deps);

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);
    let user_address = deps
        .api
        .canonical_address(&HumanAddr::from(TEST_USER))
        .unwrap();

    let config = config::read(&deps.storage).unwrap();
    let mut reward = reward::read(&deps.storage).unwrap();
    reward.total_deposit = reward.total_deposit.add(deposit_amount);
    reward::store(&mut deps.storage, &reward).unwrap();

    let mut user = user::read(&deps.storage, &user_address).unwrap();
    user.amount = deposit_amount;
    user::store(&mut deps.storage, &user_address, &user).unwrap();

    // ========================== before start
    env.block.time = TEST_POOL_START - 1;
    let msg = HandleMsg::Update { target: None };
    contract::handle(&mut deps, env.clone(), msg)
        .expect("testing: failed to execute update message");

    let reward = reward::read(&deps.storage).unwrap();
    assert_eq!(reward.last_update_time, TEST_POOL_START);
    assert_eq!(reward.reward_per_token_stored, Decimal256::zero());

    // ========================== middle of sale
    env.block.time = TEST_POOL_START + (TEST_POOL_PERIOD / 2);
    let msg = HandleMsg::Update { target: None };
    contract::handle(&mut deps, env.clone(), msg)
        .expect("testing: failed to execute update message");

    let reward = reward::read(&deps.storage).unwrap();
    assert_eq!(reward.last_update_time, env.block.time);
    assert_eq!(
        reward.reward_per_token_stored,
        Decimal256::from_uint256(Uint256::from(TEST_POOL_PERIOD / 2))
            .mul(config.distribution_config.reward_rate)
            .div(Decimal256::from_uint256(reward.total_deposit)),
    );

    // ========================== end of sale
    env.block.time = TEST_POOL_START + TEST_POOL_PERIOD + 1;
    let msg = HandleMsg::Update { target: None };
    contract::handle(&mut deps, env, msg).expect("testing: failed to execute update message");

    let reward = reward::read(&deps.storage).unwrap();
    assert_eq!(reward.last_update_time, TEST_POOL_START + TEST_POOL_PERIOD);
    assert_eq!(
        reward.reward_per_token_stored,
        Decimal256::from_uint256(Uint256::from(TEST_POOL_PERIOD))
            .mul(config.distribution_config.reward_rate)
            .div(Decimal256::from_uint256(reward.total_deposit)),
    );

    let user = user::read(&deps.storage, &user_address).unwrap();
    assert_eq!(user.reward, Uint256::zero());
    assert_eq!(user.reward_per_token_paid, Decimal256::zero());
}

#[test]
fn handle_update_with_target() {
    let mut deps = mock_dependencies(20, &[]);
    let mut env = utils::initialize(&mut deps);

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);
    let user_address = deps
        .api
        .canonical_address(&HumanAddr::from(TEST_USER))
        .unwrap();

    let config = config::read(&deps.storage).unwrap();
    let mut reward = reward::read(&deps.storage).unwrap();
    reward.total_deposit = reward.total_deposit.add(deposit_amount);
    reward::store(&mut deps.storage, &reward).unwrap();

    let mut user = user::read(&deps.storage, &user_address).unwrap();
    user.amount = deposit_amount;
    user::store(&mut deps.storage, &user_address, &user).unwrap();

    // ========================== before start
    env.block.time = TEST_POOL_START - 1;
    let msg = HandleMsg::Update {
        target: Option::from(HumanAddr::from(TEST_USER)),
    };
    contract::handle(&mut deps, env.clone(), msg)
        .expect("testing: failed to execute update message");

    let user = user::read(&deps.storage, &user_address).unwrap();
    assert_eq!(user.reward, Uint256::zero());
    assert_eq!(user.reward_per_token_paid, Decimal256::zero());

    // ========================== middle of sale
    env.block.time = TEST_POOL_START + (TEST_POOL_PERIOD / 2);
    let msg = HandleMsg::Update {
        target: Option::from(HumanAddr::from(TEST_USER)),
    };
    contract::handle(&mut deps, env.clone(), msg)
        .expect("testing: failed to execute update message");

    let user = user::read(&deps.storage, &user_address).unwrap();
    assert_eq!(user.reward, Uint256::from(TEST_POOL_PERIOD / 2));
    assert_eq!(
        user.reward_per_token_paid,
        Decimal256::from_uint256(Uint256::from(TEST_POOL_PERIOD / 2))
            .mul(config.distribution_config.reward_rate)
            .div(Decimal256::from_uint256(reward.total_deposit))
    );

    // ========================== end of sale
    env.block.time = TEST_POOL_START + TEST_POOL_PERIOD + 1;
    let msg = HandleMsg::Update {
        target: Option::from(HumanAddr::from(TEST_USER)),
    };
    contract::handle(&mut deps, env, msg).expect("testing: failed to execute update message");

    let user = user::read(&deps.storage, &user_address).unwrap();
    assert_eq!(user.reward, Uint256::from(TEST_POOL_PERIOD));
    assert_eq!(
        user.reward_per_token_paid,
        Decimal256::from_uint256(Uint256::from(TEST_POOL_PERIOD))
            .mul(config.distribution_config.reward_rate)
            .div(Decimal256::from_uint256(reward.total_deposit))
    );
}
