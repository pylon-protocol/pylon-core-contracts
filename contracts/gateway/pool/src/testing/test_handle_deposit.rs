use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Api, CosmosMsg, SubMsg, Timestamp, Uint128, WasmMsg};
use cw20::Cw20ReceiveMsg;
use pylon_gateway::pool_msg::{Cw20HookMsg, ExecuteMsg};
use std::ops::Div;
use std::str::FromStr;

use crate::contract;
use crate::state::{config, reward, user};
use crate::testing::constants::*;
use crate::testing::utils;

const DEPOSIT_AMOUNT: u64 = 1000000u64;

#[test]
fn handle_deposit() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);
    let user = mock_info(TEST_USER, &[]);

    let deposit_amount = Uint128::from(DEPOSIT_AMOUNT);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: user.sender.to_string(),
        amount: deposit_amount,
        msg: to_binary(&Cw20HookMsg::Deposit {}).unwrap(),
    });

    let token = mock_info(TEST_SHARE_TOKEN, &[]);
    let res = contract::execute(deps.as_mut(), mock_env(), token, msg)
        .expect("testing: handle deposit message");
    assert_eq!(res.data, None);
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                msg: to_binary(&ExecuteMsg::Update {
                    target: Option::Some(user.sender.to_string())
                })
                .unwrap(),

                funds: vec![]
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_CONTRACT_ADDR.to_string(),
                msg: to_binary(&ExecuteMsg::DepositInternal {
                    sender: user.sender.to_string(),
                    amount: Uint256::from(deposit_amount)
                })
                .unwrap(),
                funds: vec![]
            }))
        ]
    );
}

#[test]
fn handle_deposit_internal() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);
    let contract_self = mock_info(MOCK_CONTRACT_ADDR, &[]);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START + 1);

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);
    let user_addr = deps.api.addr_canonicalize(TEST_USER).unwrap();

    let msg = ExecuteMsg::DepositInternal {
        sender: TEST_USER.to_string(),
        amount: deposit_amount,
    };
    let res = contract::execute(deps.as_mut(), env, contract_self, msg)
        .expect("testing: handle internal deposit message");
    assert_eq!(res.data, None);
    assert_eq!(res.messages, vec![]);

    let reward = reward::read(&deps.storage).unwrap();
    assert_eq!(reward.total_deposit, deposit_amount);

    let user = user::read(&deps.storage, &user_addr).unwrap();
    assert_eq!(user.amount, deposit_amount);
}

#[test]
fn handle_deposit_internal_check_sender() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);

    let user = mock_info(TEST_USER, &[]);
    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);
    let msg = ExecuteMsg::DepositInternal {
        sender: TEST_USER.to_string(),
        amount: deposit_amount,
    };
    let err = contract::execute(deps.as_mut(), mock_env(), user, msg)
        .expect_err("testing: should accept message from contract itself");
    utils::assert_generic_err("check_sender", err);
}

#[test]
fn handle_deposit_internal_check_deposit_time() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);
    let contract_self = mock_info(MOCK_CONTRACT_ADDR, &[]);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START - 1); // should fail

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);
    let msg = ExecuteMsg::DepositInternal {
        sender: TEST_USER.to_string(),
        amount: deposit_amount,
    };
    let err = contract::execute(deps.as_mut(), env, contract_self, msg)
        .expect_err("testing: should fail if tries to execute from outside of deposit time range");
    utils::assert_generic_err("check_deposit_time", err);
}

#[test]
fn handle_deposit_internal_check_total_cap() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);
    let contract_self = mock_info(MOCK_CONTRACT_ADDR, &[]);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START + 1); // should fail

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);

    let mut config = config::read(&deps.storage).unwrap();
    config.deposit_config.total_cap = deposit_amount.div(Decimal256::from_str("2.0").unwrap());
    config::store(&mut deps.storage, &config).unwrap();

    let msg = ExecuteMsg::DepositInternal {
        sender: TEST_USER.to_string(),
        amount: deposit_amount,
    };
    let err = contract::execute(deps.as_mut(), env, contract_self, msg)
        .expect_err("testing: should fail if deposit amount exceeds total cap");
    utils::assert_generic_err("check_total_cap", err);
}

#[test]
fn handle_deposit_internal_check_user_cap() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);
    let contract_self = mock_info(MOCK_CONTRACT_ADDR, &[]);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START + 1); // should fail

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);

    let mut config = config::read(&deps.storage).unwrap();
    config.deposit_config.user_cap = deposit_amount.div(Decimal256::from_str("2.0").unwrap());
    config::store(&mut deps.storage, &config).unwrap();

    let msg = ExecuteMsg::DepositInternal {
        sender: TEST_USER.to_string(),
        amount: deposit_amount,
    };
    let err = contract::execute(deps.as_mut(), env, contract_self, msg)
        .expect_err("testing: should fail if deposit amount exceeds user cap");
    utils::assert_generic_err("check_user_cap", err);
}
