use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Api, CosmosMsg, SubMsg, Timestamp, WasmMsg};
use cw20::Cw20ExecuteMsg;
use pylon_gateway::pool_msg::ExecuteMsg;

use crate::contract;
use crate::state::{reward, user};
use crate::testing::constants::*;
use crate::testing::utils;

const WITHDRAW_AMOUNT: u64 = 1000000u64;

#[test]
fn handle_withdraw() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);
    let user = mock_info(TEST_USER, &[]);

    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let msg = ExecuteMsg::Withdraw {
        amount: withdraw_amount,
    };
    let res = contract::execute(deps.as_mut(), mock_env(), user.clone(), msg)
        .expect("testing: handle withdraw message");
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
                msg: to_binary(&ExecuteMsg::WithdrawInternal {
                    sender: user.sender.to_string(),
                    amount: withdraw_amount
                })
                .unwrap(),
                funds: vec![]
            }))
        ]
    );
}

#[test]
fn handle_withdraw_internal() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);
    let contract_self = mock_info(MOCK_CONTRACT_ADDR, &[]);
    env.block.time = Timestamp::from_seconds(0);

    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let user_addr = deps.api.addr_canonicalize(TEST_USER).unwrap();

    // store mock data
    reward::store(
        &mut deps.storage,
        &reward::Reward {
            total_deposit: withdraw_amount,
            last_update_time: 0,
            reward_per_token_stored: Default::default(),
        },
    )
    .unwrap();
    user::store(
        &mut deps.storage,
        &user_addr,
        &user::User {
            amount: withdraw_amount,
            reward: Default::default(),
            reward_per_token_paid: Default::default(),
        },
    )
    .unwrap();

    let msg = ExecuteMsg::WithdrawInternal {
        sender: TEST_USER.to_string(),
        amount: withdraw_amount,
    };
    let res = contract::execute(deps.as_mut(), mock_env(), contract_self, msg)
        .expect("testing: handle internal withdraw message");
    assert_eq!(res.data, None);
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_SHARE_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_USER.to_string(),
                amount: withdraw_amount.into(),
            })
            .unwrap(),
            funds: vec![]
        }))]
    );

    let reward = reward::read(deps.as_ref().storage).unwrap();
    assert_eq!(reward.total_deposit, Uint256::zero());

    let user = user::read(deps.as_ref().storage, &user_addr).unwrap();
    assert_eq!(user.amount, Uint256::zero());
}

#[test]
fn handle_withdraw_internal_check_sender() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);

    let user = mock_info(TEST_USER, &[]);
    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let msg = ExecuteMsg::WithdrawInternal {
        sender: TEST_USER.to_string(),
        amount: withdraw_amount,
    };
    let err = contract::execute(deps.as_mut(), mock_env(), user, msg)
        .expect_err("testing: should accept message from contract itself");
    utils::assert_generic_err("check_sender", err);
}

#[test]
fn handle_deposit_internal_check_withdraw_time() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);
    let contract_self = mock_info(MOCK_CONTRACT_ADDR, &[]);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START + 1);

    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let msg = ExecuteMsg::WithdrawInternal {
        sender: TEST_USER.to_string(),
        amount: withdraw_amount,
    };
    let err = contract::execute(deps.as_mut(), mock_env(), contract_self, msg)
        .expect_err("testing: should fail if tries to execute from outside of withdraw time range");
    utils::assert_generic_err("check_withdraw_time", err);
}

#[test]
fn handle_withdraw_internal_check_user_amount() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);
    let contract_self = mock_info(MOCK_CONTRACT_ADDR, &[]);
    env.block.time = Timestamp::from_seconds(0);

    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let msg = ExecuteMsg::WithdrawInternal {
        sender: TEST_USER.to_string(),
        amount: withdraw_amount,
    };

    let err = contract::execute(deps.as_mut(), mock_env(), contract_self, msg)
        .expect_err("testing: should fail if withdraw amount exceeds user amount");
    utils::assert_generic_err("check_user_amount", err);
}
