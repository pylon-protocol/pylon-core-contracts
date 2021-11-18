use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, to_binary, CosmosMsg, Env, MessageInfo, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use pylon_token::gov_msg::{VoteOption as GovVoteOption, VoterInfo as GovVoterInfo};
use pylon_token::gov_resp::StakerResponse;

use crate::error::ContractError;
use crate::executions::staking::withdraw_voting_tokens;
use crate::executions::ExecuteResult;
use crate::queries::bank::query_staker;
use crate::state::poll::{Poll, PollStatus, VoteOption};
use crate::testing::{mock_deps, MockDeps, TEST_VOTER, VOTING_TOKEN};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Option<Uint128>,
) -> ExecuteResult {
    withdraw_voting_tokens(deps.as_mut(), env, info, sender, amount)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    // stake
    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();

    // double the balance, only half will be withdrawn
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(22u128))],
    )]);

    // unstake - #1
    let response = exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Some(Uint128::from(11u128)),
    )
    .unwrap();

    let message = response.messages.get(0).expect("no message");
    assert_eq!(
        message,
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

    // unstake - #2
    let response = exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        None,
    )
    .unwrap();

    let message = response.messages.get(0).expect("no message");
    assert_eq!(
        message,
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
}

#[test]
fn success_with_poll() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_proposal_deposit = super::instantiate::default_msg().proposal_deposit;

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + default_proposal_deposit.u128() * 2),
        )],
    )]);

    // stake
    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();

    // create poll
    super::poll_create::default(&mut deps); // #1
    super::poll_create::default(&mut deps); // #2

    // increase balance
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(22u128 + default_proposal_deposit.u128() * 2),
        )],
    )]);

    // vote
    super::poll_cast_vote::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        1,
        VoteOption::Yes,
        Uint128::from(11u128),
    )
    .unwrap();

    super::poll_cast_vote::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        2,
        VoteOption::No,
        Uint128::from(11u128),
    )
    .unwrap();

    // pass poll
    let mut poll = Poll::load(deps.as_ref().storage, &2u64).unwrap();
    poll.status = PollStatus::Passed;
    Poll::save(deps.as_mut().storage, &2u64, &poll).unwrap();

    // withdraw
    exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Some(Uint128::from(11u128)),
    )
    .unwrap();

    // check staker query
    let response = query_staker(deps.as_ref(), mock_env(), TEST_VOTER.to_string()).unwrap();
    let response: StakerResponse = from_binary(&response).unwrap();
    assert_eq!(
        response.locked_balance,
        vec![(
            1u64,
            GovVoterInfo {
                vote: GovVoteOption::Yes,
                balance: Uint128::from(11u128)
            }
        )]
    )
}

#[test]
fn test_nothing_staked() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Some(Uint128::from(5u128)),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NothingStaked {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_invalid_withdraw_amount() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    // stake
    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();

    // withdraw
    match exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Some(Uint128::from(12u128)),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InvalidWithdrawAmount {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
