use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{coin, Api, Timestamp};
use pylon_gateway::swap_msg::ExecuteMsg;

use crate::contract;
use crate::error::ContractError;
use crate::state::{state, user};
use crate::testing::constants::{TEST_PRICE, TEST_SWAP_POOL_SIZE, TEST_USER_1, TEST_USER_2};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;
use cosmwasm_std::testing::mock_info;
use std::str::FromStr;

#[test]
fn execute_deposit_check_period() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, info) = utils::initialize(&mut deps);

    // lt start
    env.block.time = Timestamp::from_seconds(0);
    let resp = contract::execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::Deposit {},
    )
    .expect_err("testing: should execute deposit failed");
    assert!(matches!(resp, ContractError::SwapNotStarted { .. }));

    // gt start lt finish

    env.block.time = Timestamp::from_seconds(5);
    let resp = contract::execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::Deposit {},
    )
    .expect_err("testing: should execute deposit failed");
    assert!(!matches!(resp, ContractError::SwapNotStarted { .. }));
    assert!(!matches!(resp, ContractError::SwapFinished { .. }));

    // gt finish
    env.block.time = Timestamp::from_seconds(12);
    let resp = contract::execute(deps.as_mut(), env, info, ExecuteMsg::Deposit {})
        .expect_err("testing: should execute deposit failed");
    assert!(matches!(resp, ContractError::SwapFinished { .. }));
}

#[test]
fn execute_deposit_check_funds() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, mut info) = utils::initialize(&mut deps);

    // zero amount
    env.block.time = Timestamp::from_seconds(5);
    let resp = contract::execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::Deposit {},
    )
    .expect_err("testing: should execute deposit failed");
    assert!(matches!(resp, ContractError::NotAllowZeroAmount { .. }));

    // ok
    env.block.time = Timestamp::from_seconds(5);
    info.funds.push(coin(100, "uusd"));
    contract::execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::Deposit {},
    )
    .expect("testing: should execute deposit succeeded");

    // other denoms
    env.block.time = Timestamp::from_seconds(5);
    info.funds.push(coin(100, "ukrw"));
    let resp = contract::execute(deps.as_mut(), env, info, ExecuteMsg::Deposit {})
        .expect_err("testing: should execute deposit failed");
    assert!(matches!(resp, ContractError::NotAllowOtherDenoms { .. }));
}

#[test]
fn execute_deposit_check_whitelist() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);

    // whitelisted
    env.block.time = Timestamp::from_seconds(5);
    let user1 = mock_info(TEST_USER_1, &[coin(100, "uusd")]);
    contract::execute(deps.as_mut(), env.clone(), user1, ExecuteMsg::Deposit {})
        .expect("testing: should execute deposit succeeded");

    // non-whitelisted
    env.block.time = Timestamp::from_seconds(5);
    let user2 = mock_info(TEST_USER_2, &[coin(100, "uusd")]);
    let resp = contract::execute(deps.as_mut(), env, user2, ExecuteMsg::Deposit {})
        .expect_err("testing: should execute deposit failed");
    assert!(matches!(resp, ContractError::NotAllowNonWhitelisted { .. }));
}

#[test]
fn execute_deposit_check_cap() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);

    env.block.time = Timestamp::from_seconds(5);
    let user = mock_info(
        TEST_USER_1,
        &[coin(
            u128::from(
                (Uint256::from(TEST_SWAP_POOL_SIZE) * Decimal256::from_str(TEST_PRICE).unwrap())
                    + Uint256::one(),
            ),
            "uusd",
        )],
    );
    let resp = contract::execute(deps.as_mut(), env, user, ExecuteMsg::Deposit {})
        .expect_err("testing: should execute deposit failed");
    assert!(matches!(resp, ContractError::PoolSizeExceeded { .. }));
}

#[test]
fn execute_deposit() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);

    let mut before_user = user::read(
        deps.as_ref().storage,
        &deps.api.addr_canonicalize(TEST_USER_1).unwrap(),
    )
    .unwrap();
    let mut before_state = state::read(deps.as_ref().storage).load().unwrap();

    env.block.time = Timestamp::from_seconds(5);
    let user = mock_info(TEST_USER_1, &[coin(100, "uusd")]);
    contract::execute(deps.as_mut(), env, user, ExecuteMsg::Deposit {})
        .expect("testing: should execute deposit succeeded");

    before_user.swapped_in += Uint256::from(100u64);
    before_user.swapped_out += Uint256::from(100u64) / Decimal256::from_str(TEST_PRICE).unwrap();
    assert_eq!(
        user::read(
            deps.as_ref().storage,
            &deps.api.addr_canonicalize(TEST_USER_1).unwrap(),
        )
        .unwrap(),
        before_user,
    );

    before_state.total_swapped += Uint256::from(100u64) / Decimal256::from_str(TEST_PRICE).unwrap();
    assert_eq!(
        state::read(deps.as_ref().storage).load().unwrap(),
        before_state,
    )
}
