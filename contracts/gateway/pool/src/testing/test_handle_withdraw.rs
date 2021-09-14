use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Api, CosmosMsg, HumanAddr, WasmMsg};
use cw20::Cw20HandleMsg;
use pylon_gateway::pool_msg::HandleMsg;

use crate::contract;
use crate::state::{reward, user};
use crate::testing::constants::{TEST_POOL_START, TEST_SHARE_TOKEN, TEST_USER};
use crate::testing::utils;

const WITHDRAW_AMOUNT: u64 = 1000000u64;

#[test]
fn handle_withdraw() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let user = mock_env(TEST_USER, &[]);

    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let msg = HandleMsg::Withdraw {
        amount: withdraw_amount,
    };
    let res =
        contract::handle(&mut deps, user.clone(), msg).expect("testing: handle withdraw message");
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
                msg: to_binary(&HandleMsg::WithdrawInternal {
                    sender: user.message.sender,
                    amount: withdraw_amount
                })
                .unwrap(),
                send: vec![]
            })
        ]
    );
}

#[test]
fn handle_withdraw_internal() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let mut contract_self = mock_env(MOCK_CONTRACT_ADDR, &[]);
    contract_self.block.time = 0;

    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let user_addr = deps
        .api
        .canonical_address(&HumanAddr::from(TEST_USER))
        .unwrap();

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

    let msg = HandleMsg::WithdrawInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: withdraw_amount,
    };
    let res = contract::handle(&mut deps, contract_self, msg)
        .expect("testing: handle internal withdraw message");
    assert_eq!(res.data, None);
    assert_eq!(
        res.messages,
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: HumanAddr::from(TEST_SHARE_TOKEN),
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: HumanAddr::from(TEST_USER),
                amount: withdraw_amount.into(),
            })
            .unwrap(),
            send: vec![]
        })]
    );

    let reward = reward::read(&deps.storage).unwrap();
    assert_eq!(reward.total_deposit, Uint256::zero());

    let user = user::read(&deps.storage, &user_addr).unwrap();
    assert_eq!(user.amount, Uint256::zero());
}

#[test]
fn handle_withdraw_internal_check_sender() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let user = mock_env(TEST_USER, &[]);
    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let msg = HandleMsg::WithdrawInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: withdraw_amount,
    };
    let err = contract::handle(&mut deps, user, msg)
        .expect_err("testing: should accept message from contract itself");
    utils::assert_generic_err("check_sender", err);
}

#[test]
fn handle_deposit_internal_check_withdraw_time() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let mut contract_self = mock_env(MOCK_CONTRACT_ADDR, &[]);
    contract_self.block.time = TEST_POOL_START + 1;

    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let msg = HandleMsg::WithdrawInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: withdraw_amount,
    };
    let err = contract::handle(&mut deps, contract_self, msg)
        .expect_err("testing: should fail if tries to execute from outside of withdraw time range");
    utils::assert_generic_err("check_withdraw_time", err);
}

#[test]
fn handle_withdraw_internal_check_user_amount() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let mut contract_self = mock_env(MOCK_CONTRACT_ADDR, &[]);
    contract_self.block.time = 0;

    let withdraw_amount = Uint256::from(WITHDRAW_AMOUNT);
    let msg = HandleMsg::WithdrawInternal {
        sender: HumanAddr::from(TEST_USER),
        amount: withdraw_amount,
    };

    let err = contract::handle(&mut deps, contract_self, msg)
        .expect_err("testing: should fail if withdraw amount exceeds user amount");
    utils::assert_generic_err("check_user_amount", err);
}
