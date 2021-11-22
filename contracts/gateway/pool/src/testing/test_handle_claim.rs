use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Api, CosmosMsg, SubMsg, Timestamp, WasmMsg};
use cw20::Cw20ExecuteMsg;
use pylon_gateway::pool_msg::ExecuteMsg;

use crate::contract;
use crate::state::user;
use crate::testing::constants::*;
use crate::testing::utils;

#[test]
fn handle_claim() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);
    let user = mock_info(TEST_USER, &[]);

    let msg = ExecuteMsg::Claim {};
    let res = contract::execute(deps.as_mut(), mock_env(), user.clone(), msg)
        .expect("testing: handle claim message");
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
                msg: to_binary(&ExecuteMsg::ClaimInternal {
                    sender: user.sender.to_string()
                })
                .unwrap(),
                funds: vec![]
            }))
        ]
    );
}

#[test]
fn handle_claim_internal() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);
    let contract_self = mock_info(MOCK_CONTRACT_ADDR, &[]);
    env.block.time = Timestamp::from_seconds(TEST_POOL_START + TEST_POOL_CLIFF + 1);

    let reward_amount = Uint256::from(1000000u64);

    user::store(
        &mut deps.storage,
        &deps.api.addr_canonicalize(TEST_USER).unwrap(),
        &user::User {
            amount: Default::default(),
            reward: reward_amount,
            reward_per_token_paid: Default::default(),
        },
    )
    .unwrap();

    let msg = ExecuteMsg::ClaimInternal {
        sender: TEST_USER.to_string(),
    };
    let res = contract::execute(deps.as_mut(), env, contract_self, msg)
        .expect("testing: handle internal claim message");
    assert_eq!(res.data, None);
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_REWARD_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_USER.to_string(),
                amount: reward_amount.into(),
            })
            .unwrap(),
            funds: vec![]
        }))]
    );
}

#[test]
fn handle_claim_internal_check_sender() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);

    let user = mock_info(TEST_USER, &[]);
    let msg = ExecuteMsg::ClaimInternal {
        sender: TEST_USER.to_string(),
    };
    let err = contract::execute(deps.as_mut(), mock_env(), user, msg)
        .expect_err("testing: should accept message from contract itself");
    utils::assert_generic_err("check_sender", err);
}

#[test]
fn handle_claim_internal_check_claim_time() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);
    let contract_self = mock_info(MOCK_CONTRACT_ADDR, &[]);
    env.block.time = Timestamp::from_seconds(TEST_POOL_CLIFF - 1);

    let msg = ExecuteMsg::ClaimInternal {
        sender: TEST_USER.to_string(),
    };
    let err = contract::execute(deps.as_mut(), env, contract_self, msg)
        .expect_err("testing: should fail to check claim time");
    utils::assert_generic_err("check_claim_time", err);
}
