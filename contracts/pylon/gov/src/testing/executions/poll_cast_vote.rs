use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, from_binary, Env, MessageInfo, Response, Uint128};
use pylon_token::common::OrderBy;
use pylon_token::gov_msg::{VoteOption as GovVoteOption, VoterInfo as GovVoterInfo};
use pylon_token::gov_resp::{PollResponse, StakerResponse, VotersResponse, VotersResponseItem};

use crate::error::ContractError;
use crate::executions::poll::cast_vote;
use crate::executions::ExecuteResult;
use crate::queries::bank::query_staker;
use crate::queries::poll::{query_poll, query_voters};
use crate::state::poll::VoteOption;
use crate::testing::{
    mock_deps, mock_env_height, MockDeps, TEST_VOTER, TEST_VOTER_2, TEST_VOTER_3, VOTING_TOKEN,
};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    vote: VoteOption,
    amount: Uint128,
) -> ExecuteResult {
    cast_vote(deps.as_mut(), env, info, poll_id, vote, amount)
}

pub fn with_stake(
    deps: &mut MockDeps,
    poll_id: u64,
    address: String,
    vote: VoteOption,
    amount: u128,
) {
    super::staking_deposit::exec(
        deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        address.clone(),
        Uint128::from(amount),
    )
    .unwrap();

    exec(
        deps,
        mock_env(),
        mock_info(address.as_str(), &[]),
        poll_id,
        vote,
        Uint128::from(amount),
    )
    .unwrap();
}

pub fn assert_cast_vote_success(
    voter: &str,
    amount: u128,
    poll_id: u64,
    vote_option: VoteOption,
    execute_res: Response,
) {
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", poll_id.to_string()),
            attr("amount", amount.to_string()),
            attr("voter", voter),
            attr("vote_option", vote_option.to_string()),
        ]
    );
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_proposal_deposit = super::instantiate::default_msg().proposal_deposit;

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + default_proposal_deposit.u128()),
        )],
    )]);

    super::poll_create::default(&mut deps); // #1
    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();

    let response = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        1,
        VoteOption::Yes,
        Uint128::from(10u128),
    )
    .unwrap();
    assert_cast_vote_success(TEST_VOTER, 10, 1, VoteOption::Yes, response);

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(22u128 + default_proposal_deposit.u128()),
        )],
    )]);

    // check staker query
    let response = query_staker(deps.as_ref(), mock_env(), TEST_VOTER.to_string()).unwrap();
    let response: StakerResponse = from_binary(&response).unwrap();
    assert_eq!(
        response,
        StakerResponse {
            balance: Uint128::from(22u128),
            share: Uint128::from(11u128),
            locked_balance: vec![(
                1u64,
                GovVoterInfo {
                    vote: GovVoteOption::Yes,
                    balance: Uint128::from(10u128),
                }
            )],
            claimable_airdrop: vec![],
        }
    );

    // check voter query
    let response = query_voters(deps.as_ref(), 1, None, None, Some(OrderBy::Desc)).unwrap();
    let response: VotersResponse = from_binary(&response).unwrap();
    assert_eq!(
        response.voters,
        vec![VotersResponseItem {
            voter: TEST_VOTER.to_string(),
            vote: GovVoteOption::Yes,
            balance: Uint128::from(10u128)
        }]
    )
}

#[test]
fn success_with_snapshot() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_proposal_deposit = super::instantiate::default_msg().proposal_deposit;

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + default_proposal_deposit.u128()),
        )],
    )]);

    super::poll_create::default(&mut deps); // #1

    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();

    let response = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        1,
        VoteOption::Yes,
        Uint128::from(10u128),
    )
    .unwrap();
    assert_cast_vote_success(TEST_VOTER, 10, 1, VoteOption::Yes, response);

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(22u128 + default_proposal_deposit.u128()),
        )],
    )]);

    let response = query_poll(deps.as_ref(), 1).unwrap();
    let response: PollResponse = from_binary(&response).unwrap();
    assert_eq!(response.staked_amount, None);

    let end_height = response.end_height;

    //cast another vote
    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER_2.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();

    // another voter cast a vote
    let response = exec(
        &mut deps,
        mock_env_height(end_height - 9, 10000),
        mock_info(TEST_VOTER_2, &[]),
        1,
        VoteOption::Yes,
        Uint128::from(10u128),
    )
    .unwrap();
    assert_cast_vote_success(TEST_VOTER_2, 10, 1, VoteOption::Yes, response);

    let response = query_poll(deps.as_ref(), 1).unwrap();
    let response: PollResponse = from_binary(&response).unwrap();
    assert_eq!(response.staked_amount, Some(Uint128::new(22)));

    // snapshot poll will not go through
    let snap_error = super::poll_snapshot::exec(
        &mut deps,
        mock_env_height(end_height - 9, 10000),
        mock_info(TEST_VOTER_2, &[]),
        1,
    )
    .unwrap_err();
    assert_eq!(ContractError::SnapshotAlreadyOccurred {}, snap_error);

    // balance be double
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(33u128 + default_proposal_deposit.u128()),
        )],
    )]);

    // another voter cast a vote but the snapshot is already occurred
    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER_3.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();

    let response = exec(
        &mut deps,
        mock_env_height(end_height - 8, 10000),
        mock_info(TEST_VOTER_3, &[]),
        1,
        VoteOption::Yes,
        Uint128::from(10u128),
    )
    .unwrap();
    assert_cast_vote_success(TEST_VOTER_3, 10, 1, VoteOption::Yes, response);

    let response = query_poll(deps.as_ref(), 1).unwrap();
    let response: PollResponse = from_binary(&response).unwrap();
    assert_eq!(response.staked_amount, Some(Uint128::new(22)));
}

#[test]
fn fail_not_enough_staked() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_proposal_deposit = super::instantiate::default_msg().proposal_deposit;

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + default_proposal_deposit.u128()),
        )],
    )]);

    super::poll_create::default(&mut deps);
    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        1,
        VoteOption::Yes,
        Uint128::from(12u128),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InsufficientStaked {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fail_already_voted() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_proposal_deposit = super::instantiate::default_msg().proposal_deposit;

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + default_proposal_deposit.u128()),
        )],
    )]);

    super::poll_create::default(&mut deps);
    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();

    exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        1,
        VoteOption::Yes,
        Uint128::from(5u128),
    )
    .unwrap();

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        1,
        VoteOption::Yes,
        Uint128::from(5u128),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::AlreadyVoted {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fail_poll_not_found() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        1,
        VoteOption::Yes,
        Uint128::from(5u128),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollNotFound {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
