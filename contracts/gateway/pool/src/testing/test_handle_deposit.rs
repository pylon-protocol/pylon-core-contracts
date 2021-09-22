use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Api, CosmosMsg, HumanAddr, Uint128, WasmMsg};
use cw20::Cw20ReceiveMsg;
use pylon_gateway::pool_msg::{Cw20HookMsg, HandleMsg};
use std::ops::Div;
use std::str::FromStr;

use crate::contract;
use crate::state::{config, reward, user};
use crate::testing::constants::*;
use crate::testing::utils;

const DEPOSIT_AMOUNT: u64 = 1000000u64;

#[test]
fn handle_deposit() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let user = mock_env(TEST_USER, &[]);

    let deposit_amount = Uint128::from(DEPOSIT_AMOUNT);
    let msg = HandleMsg::Receive(Cw20ReceiveMsg {
        sender: user.message.sender.clone(),
        amount: deposit_amount,
        msg: Option::from(to_binary(&Cw20HookMsg::Deposit {}).unwrap()),
    });

    let token = mock_env(TEST_SHARE_TOKEN, &[]);
    let res = contract::handle(&mut deps, token, msg).expect("testing: handle deposit message");
    assert_eq!(res.data, None);
    assert_eq!(
        res.messages,
        vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr::from(MOCK_CONTRACT_ADDR),
                msg: to_binary(&HandleMsg::Update {
                    target: Option::Some(user.message.sender.clone())
                })
                .unwrap(),
                send: vec![]
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr::from(MOCK_CONTRACT_ADDR),
                msg: to_binary(&HandleMsg::DepositInternal {
                    sender: user.message.sender,
                    amount: Uint256::from(deposit_amount)
                })
                .unwrap(),
                send: vec![]
            })
        ]
    );
}

#[test]
fn handle_deposit_internal() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let mut contract_self = mock_env(MOCK_CONTRACT_ADDR, &[]);
    contract_self.block.time = TEST_POOL_START + 1;

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);
    let user_addr = deps
        .api
        .canonical_address(&HumanAddr::from(TEST_USER))
        .unwrap();

    let msg = HandleMsg::DepositInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: deposit_amount,
    };
    let res = contract::handle(&mut deps, contract_self, msg)
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
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let user = mock_env(TEST_USER, &[]);
    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);
    let msg = HandleMsg::DepositInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: deposit_amount,
    };
    let err = contract::handle(&mut deps, user, msg)
        .expect_err("testing: should accept message from contract itself");
    utils::assert_generic_err("check_sender", err);
}

#[test]
fn handle_deposit_internal_check_deposit_time() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let mut contract_self = mock_env(MOCK_CONTRACT_ADDR, &[]);
    contract_self.block.time = TEST_POOL_START - 1; // should fail

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);
    let user_addr = deps
        .api
        .canonical_address(&HumanAddr::from(TEST_USER))
        .unwrap();

    let msg = HandleMsg::DepositInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: deposit_amount,
    };
    let err = contract::handle(&mut deps, contract_self, msg)
        .expect_err("testing: should fail if tries to execute from outside of deposit time range");
    utils::assert_generic_err("check_deposit_time", err);
}

#[test]
fn handle_deposit_internal_check_total_cap() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let mut contract_self = mock_env(MOCK_CONTRACT_ADDR, &[]);
    contract_self.block.time = TEST_POOL_START + 1; // should fail

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);

    let mut config = config::read(&deps.storage).unwrap();
    config.deposit_config.total_cap = deposit_amount.div(Decimal256::from_str("2.0").unwrap());
    config::store(&mut deps.storage, &config).unwrap();

    let msg = HandleMsg::DepositInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: deposit_amount,
    };
    let err = contract::handle(&mut deps, contract_self, msg)
        .expect_err("testing: should fail if deposit amount exceeds total cap");
    utils::assert_generic_err("check_total_cap", err);
}

#[test]
fn handle_deposit_internal_check_user_cap() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let mut contract_self = mock_env(MOCK_CONTRACT_ADDR, &[]);
    contract_self.block.time = TEST_POOL_START + 1; // should fail

    let deposit_amount = Uint256::from(DEPOSIT_AMOUNT);

    let mut config = config::read(&deps.storage).unwrap();
    config.deposit_config.user_cap = deposit_amount.div(Decimal256::from_str("2.0").unwrap());
    config::store(&mut deps.storage, &config).unwrap();

    let msg = HandleMsg::DepositInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: deposit_amount,
    };
    let err = contract::handle(&mut deps, contract_self, msg)
        .expect_err("testing: should fail if deposit amount exceeds user cap");
    utils::assert_generic_err("check_user_cap", err);
}
