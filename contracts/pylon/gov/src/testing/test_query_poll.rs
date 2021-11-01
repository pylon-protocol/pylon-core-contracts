use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Uint128};
use cw20::Cw20ExecuteMsg;
use pylon_token::common::OrderBy;
use pylon_token::gov::{PollExecuteMsg, PollResponse, PollStatus, PollsResponse, QueryMsg};

use crate::contract;
use crate::error::ContractError;
use crate::testing::constants::*;
use crate::testing::message::create_poll_msg;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils::{mock_env_height, mock_instantiate};

#[test]
fn query_polls() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);

    let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(123),
    })
    .unwrap();

    let exec_msg_bz2 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(12),
    })
    .unwrap();

    let exec_msg_bz3 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(1),
    })
    .unwrap();

    let execute_msgs: Vec<PollExecuteMsg> = vec![
        PollExecuteMsg {
            order: 1u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz,
        },
        PollExecuteMsg {
            order: 3u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz3,
        },
        PollExecuteMsg {
            order: 2u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz2,
        },
    ];

    let msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        Some("http://google.com".to_string()),
        Some(execute_msgs.clone()),
    );

    let _execute_res = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    let msg = create_poll_msg("test2".to_string(), "test2".to_string(), None, None);
    let _execute_res = contract::execute(deps.as_mut(), env, info, msg).unwrap();

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: None,
            start_after: None,
            limit: None,
            order_by: Some(OrderBy::Asc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(
        response.polls,
        vec![
            PollResponse {
                id: 1u64,
                creator: TEST_CREATOR.to_string(),
                status: PollStatus::InProgress,
                end_height: 20000u64,
                title: "test".to_string(),
                description: "test".to_string(),
                link: Some("http://google.com".to_string()),
                deposit_amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
                execute_data: Some(execute_msgs.clone()),
                yes_votes: Uint128::zero(),
                no_votes: Uint128::zero(),
                staked_amount: None,
                total_balance_at_end_poll: None,
            },
            PollResponse {
                id: 2u64,
                creator: TEST_CREATOR.to_string(),
                status: PollStatus::InProgress,
                end_height: 20000u64,
                title: "test2".to_string(),
                description: "test2".to_string(),
                link: None,
                deposit_amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
                execute_data: None,
                yes_votes: Uint128::zero(),
                no_votes: Uint128::zero(),
                staked_amount: None,
                total_balance_at_end_poll: None,
            },
        ]
    );

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: None,
            start_after: Some(1u64),
            limit: None,
            order_by: Some(OrderBy::Asc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(
        response.polls,
        vec![PollResponse {
            id: 2u64,
            creator: TEST_CREATOR.to_string(),
            status: PollStatus::InProgress,
            end_height: 20000u64,
            title: "test2".to_string(),
            description: "test2".to_string(),
            link: None,
            deposit_amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            execute_data: None,
            yes_votes: Uint128::zero(),
            no_votes: Uint128::zero(),
            staked_amount: None,
            total_balance_at_end_poll: None,
        },]
    );

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: None,
            start_after: Some(2u64),
            limit: None,
            order_by: Some(OrderBy::Desc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(
        response.polls,
        vec![PollResponse {
            id: 1u64,
            creator: TEST_CREATOR.to_string(),
            status: PollStatus::InProgress,
            end_height: 20000u64,
            title: "test".to_string(),
            description: "test".to_string(),
            link: Some("http://google.com".to_string()),
            deposit_amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            execute_data: Some(execute_msgs),
            yes_votes: Uint128::zero(),
            no_votes: Uint128::zero(),
            staked_amount: None,
            total_balance_at_end_poll: None,
        }]
    );

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: Some(PollStatus::InProgress),
            start_after: Some(1u64),
            limit: None,
            order_by: Some(OrderBy::Asc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(
        response.polls,
        vec![PollResponse {
            id: 2u64,
            creator: TEST_CREATOR.to_string(),
            status: PollStatus::InProgress,
            end_height: 20000u64,
            title: "test2".to_string(),
            description: "test2".to_string(),
            link: None,
            deposit_amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            execute_data: None,
            yes_votes: Uint128::zero(),
            no_votes: Uint128::zero(),
            staked_amount: None,
            total_balance_at_end_poll: None,
        },]
    );

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: Some(PollStatus::Passed),
            start_after: None,
            limit: None,
            order_by: None,
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls, vec![]);
}

#[test]
fn poll_not_found() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let res = contract::query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 });

    match res {
        Err(ContractError::PollNotFound {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
        _ => panic!("Must return error"),
    }
}
