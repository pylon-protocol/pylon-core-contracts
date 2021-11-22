use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Api, CosmosMsg, Env, MessageInfo, Response, SubMsg,
    Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use pylon_token::common::OrderBy;
use pylon_token::gov_resp::{PollResponse, PollsResponse, StakerResponse, VotersResponse};
use terraswap::querier::query_token_balance;

use crate::error::ContractError;
use crate::executions::poll::end;
use crate::executions::ExecuteResult;
use crate::queries::bank::query_staker;
use crate::queries::poll::{query_poll, query_polls_with_status_filter, query_voters};
use crate::state::bank::TokenManager;
use crate::state::poll::{PollStatus, VoteOption, VoterInfo};
use crate::testing::{
    mock_deps, mock_env_height, MockDeps, TEST_CREATOR, TEST_VOTER, TEST_VOTER_2, VOTING_TOKEN,
};

pub fn exec(deps: &mut MockDeps, env: Env, _info: MessageInfo, poll_id: u64) -> ExecuteResult {
    end(deps.as_mut(), env, poll_id)
}

pub fn default(deps: &mut MockDeps, end_height: u64, poll_id: u64) -> (Env, MessageInfo, Response) {
    let env = mock_env_height(end_height, 0);
    let info = mock_info(TEST_CREATOR, &[]);

    let response = exec(deps, env.clone(), info.clone(), poll_id).unwrap();

    (env, info, response)
}

pub fn assert_end_poll_success(
    deps: &MockDeps,
    response: Response,
    poll_id: u64,
    proposal_deposit: Uint128,
) {
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", poll_id.to_string()),
            attr("rejected_reason", ""),
            attr("passed", "true"),
        ]
    );
    assert_eq!(
        response.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_CREATOR.to_string(),
                amount: proposal_deposit,
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    // check directly
    let response = query_poll(deps.as_ref(), poll_id).unwrap();
    let response: PollResponse = from_binary(&response).unwrap();
    assert_eq!(response.status, PollStatus::Passed.into());

    // check rejected polls
    let response = query_polls_with_status_filter(
        deps.as_ref(),
        Some(PollStatus::Rejected),
        None,
        None,
        Some(OrderBy::Desc),
    )
    .unwrap();
    let response: PollsResponse = from_binary(&response).unwrap();
    assert_eq!(response.polls.len(), 0);

    // check ongoing polls
    let response = query_polls_with_status_filter(
        deps.as_ref(),
        Some(PollStatus::InProgress),
        None,
        None,
        Some(OrderBy::Desc),
    )
    .unwrap();
    let response: PollsResponse = from_binary(&response).unwrap();
    assert_eq!(response.polls.len(), 0);

    // check passed polls
    let response = query_polls_with_status_filter(
        deps.as_ref(),
        Some(PollStatus::Passed),
        None,
        None,
        Some(OrderBy::Desc),
    )
    .unwrap();
    let response: PollsResponse = from_binary(&response).unwrap();
    assert_eq!(response.polls.len(), 1);
}

pub fn assert_end_poll_fail(
    deps: &MockDeps,
    response: Response,
    poll_id: u64,
    rejected_reason: &str,
) {
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", poll_id.to_string()),
            attr("rejected_reason", rejected_reason.to_string()),
            attr("passed", "false"),
        ]
    );

    // check directly
    let response = query_poll(deps.as_ref(), poll_id).unwrap();
    let response: PollResponse = from_binary(&response).unwrap();
    assert_eq!(response.status, PollStatus::Rejected.into());

    // check rejected polls
    let response = query_polls_with_status_filter(
        deps.as_ref(),
        Some(PollStatus::Rejected),
        None,
        None,
        Some(OrderBy::Desc),
    )
    .unwrap();
    let response: PollsResponse = from_binary(&response).unwrap();
    assert_eq!(response.polls.len(), 1);

    // check ongoing polls
    let response = query_polls_with_status_filter(
        deps.as_ref(),
        Some(PollStatus::InProgress),
        None,
        None,
        Some(OrderBy::Desc),
    )
    .unwrap();
    let response: PollsResponse = from_binary(&response).unwrap();
    assert_eq!(response.polls.len(), 0);

    // check passed polls
    let response = query_polls_with_status_filter(
        deps.as_ref(),
        Some(PollStatus::Passed),
        None,
        None,
        Some(OrderBy::Desc),
    )
    .unwrap();
    let response: PollsResponse = from_binary(&response).unwrap();
    assert_eq!(response.polls.len(), 0);
}

#[test]
fn end_poll() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_init_msg = super::instantiate::default_msg();

    // create poll
    const POLL_ID: u64 = 1;
    const STAKE_AMOUNT: u128 = 1000;
    let (env, _, _) = super::poll_create::default(&mut deps); // #1

    let proposal_deposit = default_init_msg.proposal_deposit.u128();
    let end_height = env.block.height + default_init_msg.voting_period;

    // stake
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(STAKE_AMOUNT + proposal_deposit),
        )],
    )]);

    super::poll_cast_vote::with_stake(
        &mut deps,
        POLL_ID,
        TEST_VOTER.to_string(),
        VoteOption::Yes,
        STAKE_AMOUNT,
    );

    let response = exec(
        &mut deps,
        mock_env_height(end_height, 0),
        mock_info(TEST_CREATOR, &[]),
        POLL_ID,
    )
    .unwrap();
    assert_end_poll_success(&deps, response, POLL_ID, Uint128::from(proposal_deposit));

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(STAKE_AMOUNT),
        )],
    )]);

    // voter info must be deleted
    let response = query_voters(deps.as_ref(), POLL_ID, None, None, None).unwrap();
    let response: VotersResponse = from_binary(&response).unwrap();
    assert_eq!(response.voters.len(), 0);

    // staker locked token must be disappeared
    let response = query_staker(deps.as_ref(), mock_env(), TEST_VOTER.to_string()).unwrap();
    let response: StakerResponse = from_binary(&response).unwrap();
    assert_eq!(
        response,
        StakerResponse {
            balance: Uint128::from(STAKE_AMOUNT),
            share: Uint128::from(STAKE_AMOUNT),
            locked_balance: vec![],
            claimable_airdrop: vec![],
        }
    );

    // But the data is still in the store
    let voter_addr_raw = deps.api.addr_canonicalize(TEST_VOTER).unwrap();
    let voter = VoterInfo::load(&deps.storage, &POLL_ID, &voter_addr_raw).unwrap();
    assert_eq!(
        voter,
        VoterInfo {
            vote: VoteOption::Yes,
            balance: Uint128::from(STAKE_AMOUNT),
        }
    );

    let token_manager = TokenManager::load(&deps.storage, &voter_addr_raw).unwrap();
    assert_eq!(
        token_manager.locked_balance,
        vec![(
            POLL_ID,
            VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(STAKE_AMOUNT),
            }
        )]
    );
}

#[test]
fn end_poll_with_controlled_quorum() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_init_msg = super::instantiate::default_msg();

    // create poll
    const STAKE_AMOUNT: u128 = 1000;
    const POLL_ID: u64 = 1;
    let (env, _, _) = super::poll_create::default(&mut deps);

    let proposal_deposit = default_init_msg.proposal_deposit.u128();
    let end_height = env.block.height + default_init_msg.voting_period;

    // voter 1
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(STAKE_AMOUNT + proposal_deposit),
        )],
    )]);

    super::poll_cast_vote::with_stake(
        &mut deps,
        POLL_ID,
        TEST_VOTER.to_string(),
        VoteOption::Yes,
        STAKE_AMOUNT,
    );

    // voter 2
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(10 * STAKE_AMOUNT + proposal_deposit),
        )],
    )]);

    super::poll_cast_vote::with_stake(
        &mut deps,
        POLL_ID,
        TEST_VOTER_2.to_string(),
        VoteOption::Yes,
        8 * STAKE_AMOUNT,
    );

    // quorum must reach
    let response = exec(
        &mut deps,
        mock_env_height(end_height, 0),
        mock_info(TEST_CREATOR, &[]),
        POLL_ID,
    )
    .unwrap();
    assert_end_poll_success(&deps, response, POLL_ID, Uint128::from(proposal_deposit));

    // check poll query
    let response = query_poll(deps.as_ref(), POLL_ID).unwrap();
    let response: PollResponse = from_binary(&response).unwrap();
    assert_eq!(
        10 * STAKE_AMOUNT,
        response.total_balance_at_end_poll.unwrap().u128()
    );
    assert_eq!(response.yes_votes.u128(), 9 * STAKE_AMOUNT);

    // actual staked amount is 10 times bigger than staked amount
    let actual_staked_weight = query_token_balance(
        &deps.as_ref().querier,
        Addr::unchecked(VOTING_TOKEN),
        Addr::unchecked(MOCK_CONTRACT_ADDR),
    )
    .unwrap()
    .checked_sub(Uint128::from(proposal_deposit))
    .unwrap();
    assert_eq!(actual_staked_weight.u128(), (10 * STAKE_AMOUNT))
}

#[test]
fn end_poll_zero_quorum() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_init_msg = super::instantiate::default_msg();

    // create poll
    const STAKE_AMOUNT: u128 = 1000;
    const POLL_ID: u64 = 1;
    let (env, _, _) = super::poll_create::default(&mut deps);

    let proposal_deposit = default_init_msg.proposal_deposit.u128();
    let end_height = env.block.height + default_init_msg.voting_period;

    // stake only
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(STAKE_AMOUNT + proposal_deposit),
        )],
    )]);

    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(STAKE_AMOUNT),
    )
    .unwrap();

    let response = exec(
        &mut deps,
        mock_env_height(end_height, 0),
        mock_info(TEST_CREATOR, &[]),
        POLL_ID,
    )
    .unwrap();
    assert_end_poll_fail(&deps, response, POLL_ID, "Quorum not reached");
}

#[test]
fn end_poll_quorum_rejected() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_init_msg = super::instantiate::default_msg();

    // create poll
    const STAKE_AMOUNT: u128 = 1000;
    const POLL_ID: u64 = 1;
    let (env, _, _) = super::poll_create::default(&mut deps);

    let proposal_deposit = default_init_msg.proposal_deposit.u128();
    let end_height = env.block.height + default_init_msg.voting_period;

    // stake
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(100 * STAKE_AMOUNT + proposal_deposit),
        )],
    )]);

    super::poll_cast_vote::with_stake(
        &mut deps,
        POLL_ID,
        TEST_VOTER.to_string(),
        VoteOption::Yes,
        STAKE_AMOUNT,
    );

    let response = exec(
        &mut deps,
        mock_env_height(end_height, 0),
        mock_info(TEST_CREATOR, &[]),
        POLL_ID,
    )
    .unwrap();
    assert_end_poll_fail(&deps, response, POLL_ID, "Quorum not reached");
}

#[test]
fn end_poll_quorum_rejected_nothing_staked() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_init_msg = super::instantiate::default_msg();

    // create poll
    const POLL_ID: u64 = 1;
    let (env, _, _) = super::poll_create::default(&mut deps);

    let end_height = env.block.height + default_init_msg.voting_period;

    let response = exec(
        &mut deps,
        mock_env_height(end_height, 0),
        mock_info(TEST_CREATOR, &[]),
        POLL_ID,
    )
    .unwrap();
    assert_end_poll_fail(&deps, response, POLL_ID, "Quorum not reached");
}

#[test]
fn end_poll_nay_rejected() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_init_msg = super::instantiate::default_msg();

    // create poll
    const STAKE_AMOUNT: u128 = 1000;
    const POLL_ID: u64 = 1;
    let (env, _, _) = super::poll_create::default(&mut deps);

    let proposal_deposit = default_init_msg.proposal_deposit.u128();
    let end_height = env.block.height + default_init_msg.voting_period;

    // stake
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(STAKE_AMOUNT + proposal_deposit),
        )],
    )]);

    super::poll_cast_vote::with_stake(
        &mut deps,
        POLL_ID,
        TEST_VOTER.to_string(),
        VoteOption::No,
        STAKE_AMOUNT,
    );

    let response = exec(
        &mut deps,
        mock_env_height(end_height, 0),
        mock_info(TEST_CREATOR, &[]),
        POLL_ID,
    )
    .unwrap();
    assert_end_poll_fail(&deps, response, POLL_ID, "Threshold not reached");
}

#[test]
fn fails_end_poll_before_end_height() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);
    super::poll_create::default(&mut deps);

    match exec(&mut deps, mock_env(), mock_info(TEST_CREATOR, &[]), 1) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollVotingPeriod {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

// #[test]
// fn fails_end_poll_quorum_inflation_without_snapshot_poll() {
//     const POLL_START_HEIGHT: u64 = 1000;
//     const POLL_ID: u64 = 1;
//     let stake_amount = 1000;
//
//     let mut deps = mock_dependencies(&coins(1000, VOTING_TOKEN));
//     mock_instantiate(deps.as_mut());
//
//     let mut creator_env = mock_env_height(POLL_START_HEIGHT, 10000);
//     let mut creator_info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));
//
//     let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
//         amount: Uint128::new(123),
//     })
//     .unwrap();
//
//     //add two messages
//     let execute_msgs: Vec<PollExecuteMsg> = vec![
//         PollExecuteMsg {
//             order: 1u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz.clone(),
//         },
//         PollExecuteMsg {
//             order: 2u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz,
//         },
//     ];
//
//     let msg = create_poll_msg(None, None, None, None, Some(execute_msgs));
//     let execute_res = entrypoints::execute(
//         deps.as_mut(),
//         creator_env.clone(),
//         creator_info.clone(),
//         msg,
//     )
//     .unwrap();
//
//     assert_create_poll_result(
//         1,
//         creator_env.block.height + DEFAULT_VOTING_PERIOD,
//         TEST_CREATOR,
//         execute_res,
//         deps.as_ref(),
//     );
//
//     deps.querier.with_token_balances(&[(
//         &VOTING_TOKEN.to_string(),
//         &[(
//             &MOCK_CONTRACT_ADDR.to_string(),
//             &Uint128::from((stake_amount + DEFAULT_PROPOSAL_DEPOSIT) as u128),
//         )],
//     )]);
//
//     let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//         sender: TEST_VOTER.to_string(),
//         amount: Uint128::from(stake_amount as u128),
//         msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
//     });
//
//     let info = mock_info(VOTING_TOKEN, &[]);
//     let execute_res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//     assert_stake_tokens_result(
//         stake_amount,
//         DEFAULT_PROPOSAL_DEPOSIT,
//         stake_amount,
//         1,
//         execute_res,
//         deps.as_ref(),
//     );
//
//     let msg = ExecuteMsg::Poll(PollMsg::CastVote {
//         poll_id: 1,
//         vote: VoteOption::Yes.into(),
//         amount: Uint128::from(stake_amount),
//     });
//     let env = mock_env_height(POLL_START_HEIGHT, 10000);
//     let info = mock_info(TEST_VOTER, &[]);
//     let execute_res = entrypoints::execute(deps.as_mut(), env, info, msg).unwrap();
//
//     assert_eq!(
//         execute_res.attributes,
//         vec![
//             attr("action", "cast_vote"),
//             attr("poll_id", POLL_ID.to_string().as_str()),
//             attr("amount", "1000"),
//             attr("voter", TEST_VOTER),
//             attr("vote_option", "yes"),
//         ]
//     );
//
//     creator_env.block.height += DEFAULT_VOTING_PERIOD - 10;
//
//     // did not SnapshotPoll
//
//     // staked amount get increased 10 times
//     deps.querier.with_token_balances(&[(
//         &VOTING_TOKEN.to_string(),
//         &[(
//             &MOCK_CONTRACT_ADDR.to_string(),
//             &Uint128::from(((10 * stake_amount) + DEFAULT_PROPOSAL_DEPOSIT) as u128),
//         )],
//     )]);
//
//     //cast another vote
//     let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//         sender: TEST_VOTER_2.to_string(),
//         amount: Uint128::from(8 * stake_amount as u128),
//         msg: to_binary(&Cw20HookMsg::Stake {}).unwrap(),
//     });
//
//     let info = mock_info(VOTING_TOKEN, &[]);
//     let _execute_res = entrypoints::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//
//     // another voter cast a vote
//     let msg = ExecuteMsg::Poll(PollMsg::CastVote {
//         poll_id: 1,
//         vote: VoteOption::Yes.into(),
//         amount: Uint128::from(stake_amount),
//     });
//     let env = mock_env_height(creator_env.block.height, 10000);
//     let info = mock_info(TEST_VOTER_2, &[]);
//     let execute_res = entrypoints::execute(deps.as_mut(), env, info, msg).unwrap();
//
//     assert_eq!(
//         execute_res.attributes,
//         vec![
//             attr("action", "cast_vote"),
//             attr("poll_id", POLL_ID.to_string().as_str()),
//             attr("amount", "1000"),
//             attr("voter", TEST_VOTER_2),
//             attr("vote_option", "yes"),
//         ]
//     );
//
//     creator_info.sender = Addr::unchecked(TEST_CREATOR);
//     creator_env.block.height += 10;
//
//     // quorum must reach
//     let msg = ExecuteMsg::Poll(PollMsg::End { poll_id: 1 });
//     let execute_res = entrypoints::execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();
//
//     assert_eq!(
//         execute_res.attributes,
//         vec![
//             attr("action", "end_poll"),
//             attr("poll_id", "1"),
//             attr("rejected_reason", "Quorum not reached"),
//             attr("passed", "false"),
//         ]
//     );
//
//     let res = entrypoints::query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
//     let value: PollResponse = from_binary(&res).unwrap();
//     assert_eq!(
//         10 * stake_amount,
//         value.total_balance_at_end_poll.unwrap().u128()
//     );
// }
