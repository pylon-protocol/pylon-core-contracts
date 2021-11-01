use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{coins, from_binary, to_binary, Uint128};
use cw20::Cw20ReceiveMsg;
use pylon_token::common::OrderBy;
use pylon_token::gov::{
    Cw20HookMsg, ExecuteMsg, PollMsg, PollResponse, QueryMsg, StakerResponse, VoteOption,
    VoterInfo, VotersResponse, VotersResponseItem,
};

use crate::contract;
use crate::error::ContractError;
use crate::testing::assert::{
    assert_cast_vote_success, assert_create_poll_result, assert_stake_tokens_result,
};
use crate::testing::constants::*;
use crate::testing::message::create_poll_msg;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils::{mock_env_height, mock_instantiate};

#[test]
fn cast_vote() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let env = mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);
    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_create_poll_result(
        1,
        DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        11,
        DEFAULT_PROPOSAL_DEPOSIT,
        11,
        1,
        execute_res,
        deps.as_ref(),
    );

    let env = mock_env_height(0, 10000);
    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));
    let amount = 10u128;
    let msg = ExecuteMsg::Poll(PollMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(amount),
    });

    let execute_res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_cast_vote_success(TEST_VOTER, amount, 1, VoteOption::Yes, execute_res);

    // balance be double
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(22u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    // Query staker
    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Staker {
            address: TEST_VOTER.to_string(),
        },
    )
    .unwrap();
    let response: StakerResponse = from_binary(&res).unwrap();
    assert_eq!(
        response,
        StakerResponse {
            balance: Uint128::from(22u128),
            share: Uint128::from(11u128),
            locked_balance: vec![(
                1u64,
                VoterInfo {
                    vote: VoteOption::Yes,
                    balance: Uint128::from(amount),
                }
            )]
        }
    );

    // Query voters
    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Voters {
            poll_id: 1u64,
            start_after: None,
            limit: None,
            order_by: Some(OrderBy::Desc),
        },
    )
    .unwrap();
    let response: VotersResponse = from_binary(&res).unwrap();
    assert_eq!(
        response.voters,
        vec![VotersResponseItem {
            voter: TEST_VOTER.to_string(),
            vote: VoteOption::Yes,
            balance: Uint128::from(amount),
        }]
    );

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Voters {
            poll_id: 1u64,
            start_after: Some(TEST_VOTER.to_string()),
            limit: None,
            order_by: None,
        },
    )
    .unwrap();
    let response: VotersResponse = from_binary(&res).unwrap();
    assert_eq!(response.voters.len(), 0);
}

#[test]
fn cast_vote_with_snapshot() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let env = mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);
    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_create_poll_result(
        1,
        DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        11,
        DEFAULT_PROPOSAL_DEPOSIT,
        11,
        1,
        execute_res,
        deps.as_ref(),
    );

    //cast_vote without snapshot
    let env = mock_env_height(0, 10000);
    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));
    let amount = 10u128;

    let msg = ExecuteMsg::Poll(PollMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(amount),
    });

    let execute_res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_cast_vote_success(TEST_VOTER, amount, 1, VoteOption::Yes, execute_res);

    // balance be double
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(22u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let res = contract::query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();
    assert_eq!(value.staked_amount, None);
    let end_height = value.end_height;

    //cast another vote
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER_2.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let _execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // another voter cast a vote
    let msg = ExecuteMsg::Poll(PollMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(10u128),
    });
    let env = mock_env_height(end_height - 9, 10000);
    let info = mock_info(TEST_VOTER_2, &[]);
    let execute_res = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_cast_vote_success(TEST_VOTER_2, amount, 1, VoteOption::Yes, execute_res);

    let res = contract::query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();
    assert_eq!(value.staked_amount, Some(Uint128::new(22)));

    // snanpshot poll will not go through
    let snap_error = contract::execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::Poll(PollMsg::Snapshot { poll_id: 1 }),
    )
    .unwrap_err();
    assert_eq!(ContractError::SnapshotAlreadyOccurred {}, snap_error);

    // balance be double
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(33u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    // another voter cast a vote but the snapshot is already occurred
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER_3.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let _execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = ExecuteMsg::Poll(PollMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(10u128),
    });
    let env = mock_env_height(end_height - 8, 10000);
    let info = mock_info(TEST_VOTER_3, &[]);
    let execute_res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_cast_vote_success(TEST_VOTER_3, amount, 1, VoteOption::Yes, execute_res);

    let res = contract::query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();
    assert_eq!(value.staked_amount, Some(Uint128::new(22)));
}

#[test]
fn fails_cast_vote_not_enough_staked() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let env = mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_create_poll_result(
        1,
        DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(10u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(10u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        10,
        DEFAULT_PROPOSAL_DEPOSIT,
        10,
        1,
        execute_res,
        deps.as_ref(),
    );

    let env = mock_env_height(0, 10000);
    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));
    let msg = ExecuteMsg::Poll(PollMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(11u128),
    });

    let res = contract::execute(deps.as_mut(), env, info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InsufficientStaked {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fails_cast_vote_twice() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let env = mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);
    let execute_res = contract::execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    assert_create_poll_result(
        1,
        env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        11,
        DEFAULT_PROPOSAL_DEPOSIT,
        11,
        1,
        execute_res,
        deps.as_ref(),
    );

    let amount = 1u128;
    let msg = ExecuteMsg::Poll(PollMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(amount),
    });
    let env = mock_env_height(0, 10000);
    let info = mock_info(TEST_VOTER, &[]);
    let execute_res = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_cast_vote_success(TEST_VOTER, amount, 1, VoteOption::Yes, execute_res);

    let msg = ExecuteMsg::Poll(PollMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(amount),
    });
    let res = contract::execute(deps.as_mut(), env, info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::AlreadyVoted {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fails_cast_vote_without_poll() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::Poll(PollMsg::CastVote {
        poll_id: 0,
        vote: VoteOption::Yes,
        amount: Uint128::from(1u128),
    });
    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));

    let res = contract::execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollNotFound {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
