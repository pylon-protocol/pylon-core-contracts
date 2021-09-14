use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Api, CosmosMsg, HumanAddr, WasmMsg};
use cw20::Cw20HandleMsg;
use pylon_gateway::pool_msg::HandleMsg;

use crate::contract;
use crate::state::user;
use crate::testing::constants::{TEST_POOL_CLIFF, TEST_REWARD_TOKEN, TEST_USER};
use crate::testing::utils;

#[test]
fn handle_claim() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let user = mock_env(TEST_USER, &[]);

    let msg = HandleMsg::Claim {};
    let res =
        contract::handle(&mut deps, user.clone(), msg).expect("testing: handle claim message");
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
                msg: to_binary(&HandleMsg::ClaimInternal {
                    sender: user.message.sender
                })
                .unwrap(),
                send: vec![]
            })
        ]
    );
}

#[test]
fn handle_claim_internal() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let mut contract_self = mock_env(MOCK_CONTRACT_ADDR, &[]);
    contract_self.block.time = TEST_POOL_CLIFF + 1;

    let reward_amount = Uint256::from(1000000u64);

    user::store(
        &mut deps.storage,
        &deps
            .api
            .canonical_address(&HumanAddr::from(TEST_USER))
            .unwrap(),
        &user::User {
            amount: Default::default(),
            reward: reward_amount,
            reward_per_token_paid: Default::default(),
        },
    )
    .unwrap();

    let msg = HandleMsg::ClaimInternal {
        sender: HumanAddr::from(TEST_USER),
    };
    let res = contract::handle(&mut deps, contract_self, msg)
        .expect("testing: handle internal claim message");
    assert_eq!(res.data, None);
    assert_eq!(
        res.messages,
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: HumanAddr::from(TEST_REWARD_TOKEN),
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: HumanAddr::from(TEST_USER),
                amount: reward_amount.into(),
            })
            .unwrap(),
            send: vec![]
        })]
    );
}

#[test]
fn handle_claim_internal_check_sender() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let user = mock_env(TEST_USER, &[]);
    let msg = HandleMsg::ClaimInternal {
        sender: HumanAddr::from(TEST_USER),
    };
    let err = contract::handle(&mut deps, user, msg)
        .expect_err("testing: should accept message from contract itself");
    utils::assert_generic_err("check_sender", err);
}

#[test]
fn handle_claim_internal_check_claim_time() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);
    let mut contract_self = mock_env(MOCK_CONTRACT_ADDR, &[]);
    contract_self.block.time = TEST_POOL_CLIFF - 1;

    let msg = HandleMsg::ClaimInternal {
        sender: HumanAddr::from(TEST_USER),
    };
    let err = contract::handle(&mut deps, contract_self, msg).expect_err("testing: should fail ");
    utils::assert_generic_err("check_claim_time", err);
}
