use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{coins, to_binary, Api, CanonicalAddr, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use pylon_token::gov_msg::{
    Cw20HookMsg, ExecuteMsg, PollStatus, StakingMsg, VoteOption, VoterInfo,
};

use crate::contract;
use crate::error::ContractError;
use crate::state::bank::{bank_r, bank_w, TokenManager};
use crate::state::poll::{poll_voter_r, poll_voter_w, poll_w, Poll};
use crate::state::state::{state_r, State};
use crate::testing::assert::assert_stake_tokens_result;
use crate::testing::constants::*;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils::mock_instantiate;

#[test]
fn withdraw_voting_tokens() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(11, 0, 11, 0, execute_res, deps.as_ref());

    let state = state_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::from(11u128),
            total_deposit: Uint128::zero(),
        }
    );

    // double the balance, only half will be withdrawn
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(22u128))],
    )]);

    let info = mock_info(TEST_VOTER, &[]);
    let msg = ExecuteMsg::Staking(StakingMsg::Unstake {
        amount: Some(Uint128::from(11u128)),
    });

    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = execute_res.messages.get(0).expect("no message");

    assert_eq!(
        msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_VOTER.to_string(),
                amount: Uint128::from(11u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    let state = state_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::from(6u128),
            total_deposit: Uint128::zero(),
        }
    );
}

#[test]
fn withdraw_voting_tokens_all() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(11, 0, 11, 0, execute_res, deps.as_ref());

    let state = state_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::from(11u128),
            total_deposit: Uint128::zero(),
        }
    );

    // double the balance, all balance withdrawn
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(22u128))],
    )]);

    let info = mock_info(TEST_VOTER, &[]);
    let msg = ExecuteMsg::Staking(StakingMsg::Unstake { amount: None });

    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = execute_res.messages.get(0).expect("no message");

    assert_eq!(
        msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_VOTER.to_string(),
                amount: Uint128::from(22u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    let state = state_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::zero(),
            total_deposit: Uint128::zero(),
        }
    );
}

#[test]
fn withdraw_voting_tokens_remove_not_in_progress_poll_voter_info() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(11, 0, 11, 0, execute_res, deps.as_ref());

    // make fake polls; one in progress & one in passed
    poll_w(&mut deps.storage)
        .save(
            &1u64.to_be_bytes(),
            &Poll {
                id: 1u64,
                creator: CanonicalAddr::from(vec![]),
                status: PollStatus::InProgress,
                yes_votes: Uint128::zero(),
                no_votes: Uint128::zero(),
                end_height: 0u64,
                title: "title".to_string(),
                category: "category".to_string(),
                description: "description".to_string(),
                deposit_amount: Uint128::zero(),
                link: None,
                execute_data: None,
                total_balance_at_end_poll: None,
                staked_amount: None,
            },
        )
        .unwrap();

    poll_w(&mut deps.storage)
        .save(
            &2u64.to_be_bytes(),
            &Poll {
                id: 1u64,
                creator: CanonicalAddr::from(vec![]),
                status: PollStatus::Passed,
                yes_votes: Uint128::zero(),
                no_votes: Uint128::zero(),
                end_height: 0u64,
                title: "title".to_string(),
                category: "category".to_string(),
                description: "description".to_string(),
                deposit_amount: Uint128::zero(),
                link: None,
                execute_data: None,
                total_balance_at_end_poll: None,
                staked_amount: None,
            },
        )
        .unwrap();

    let voter_addr_raw = deps.api.addr_canonicalize(TEST_VOTER).unwrap();
    poll_voter_w(&mut deps.storage, 1u64)
        .save(
            voter_addr_raw.as_slice(),
            &VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(5u128),
            },
        )
        .unwrap();
    poll_voter_w(&mut deps.storage, 2u64)
        .save(
            voter_addr_raw.as_slice(),
            &VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(5u128),
            },
        )
        .unwrap();
    bank_w(&mut deps.storage)
        .save(
            voter_addr_raw.as_slice(),
            &TokenManager {
                share: Uint128::from(11u128),
                locked_balance: vec![
                    (
                        1u64,
                        VoterInfo {
                            vote: VoteOption::Yes,
                            balance: Uint128::from(5u128),
                        },
                    ),
                    (
                        2u64,
                        VoterInfo {
                            vote: VoteOption::Yes,
                            balance: Uint128::from(5u128),
                        },
                    ),
                ],
            },
        )
        .unwrap();

    // withdraw voting token must remove not in-progress votes infos from the store
    let info = mock_info(TEST_VOTER, &[]);
    let msg = ExecuteMsg::Staking(StakingMsg::Unstake {
        amount: Some(Uint128::from(5u128)),
    });

    let _ = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let voter = poll_voter_r(&deps.storage, 1u64)
        .load(voter_addr_raw.as_slice())
        .unwrap();
    assert_eq!(
        voter,
        VoterInfo {
            vote: VoteOption::Yes,
            balance: Uint128::from(5u128),
        }
    );
    assert!(poll_voter_r(&deps.storage, 2u64)
        .load(voter_addr_raw.as_slice())
        .is_err(),);

    let token_manager = bank_r(&deps.storage)
        .load(voter_addr_raw.as_slice())
        .unwrap();
    assert_eq!(
        token_manager.locked_balance,
        vec![(
            1u64,
            VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(5u128),
            }
        )]
    );
}

#[test]
fn fails_withdraw_voting_tokens_no_stake() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));
    let msg = ExecuteMsg::Staking(StakingMsg::Unstake {
        amount: Some(Uint128::from(11u128)),
    });

    let res = contract::execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NothingStaked {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fails_withdraw_too_many_tokens() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(10u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(10u128),
        msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(10, 0, 10, 0, execute_res, deps.as_ref());

    let info = mock_info(TEST_VOTER, &[]);
    let msg = ExecuteMsg::Staking(StakingMsg::Unstake {
        amount: Some(Uint128::from(11u128)),
    });

    let res = contract::execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InvalidWithdrawAmount {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
