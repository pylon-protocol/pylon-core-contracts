use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, from_binary, to_binary, Uint128};
use cw20::Cw20ReceiveMsg;
use pylon_token::gov::{Cw20HookMsg, ExecuteMsg, QueryMsg, StakerResponse, StakingMsg};

use crate::contract;
use crate::error::ContractError;
use crate::testing::assert::assert_stake_tokens_result;
use crate::testing::constants::*;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils::mock_instantiate;

#[test]
fn stake_voting_tokens() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(11, 0, 11, 0, execute_res, deps.as_ref());
}

#[test]
fn fails_insufficient_funds() {
    let mut deps = mock_dependencies(&[]);

    // initialize the store
    mock_instantiate(deps.as_mut());

    // insufficient token
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(0u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let res = contract::execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InsufficientFunds {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fails_staking_wrong_token() {
    let mut deps = mock_dependencies(&[]);

    // initialize the store
    mock_instantiate(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    // wrong token
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(&(VOTING_TOKEN.to_string() + "2"), &[]);
    let res = contract::execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn share_calculation() {
    let mut deps = mock_dependencies(&[]);

    // initialize the store
    mock_instantiate(deps.as_mut());

    // create 100 share
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let _res = contract::execute(deps.as_mut(), mock_env(), info, msg);

    // add more balance(100) to make share:balance = 1:2
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(200u128 + 100u128),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "staking"),
            attr("sender", TEST_VOTER),
            attr("share", "50"),
            attr("amount", "100"),
        ]
    );

    let msg = ExecuteMsg::Staking(StakingMsg::Withdraw {
        amount: Some(Uint128::from(100u128)),
    });
    let info = mock_info(TEST_VOTER, &[]);
    let res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "withdraw"),
            attr("recipient", TEST_VOTER),
            attr("amount", "100"),
        ]
    );

    // 100 tokens withdrawn
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(200u128))],
    )]);

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Staker {
            address: TEST_VOTER.to_string(),
        },
    )
    .unwrap();
    let stake_info: StakerResponse = from_binary(&res).unwrap();
    assert_eq!(stake_info.share, Uint128::new(100));
    assert_eq!(stake_info.balance, Uint128::new(200));
    assert_eq!(stake_info.locked_balance, vec![]);
}
