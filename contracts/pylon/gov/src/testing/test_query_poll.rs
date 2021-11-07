use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, Uint128};
use cw20::Cw20ExecuteMsg;
use pylon_token::common::OrderBy;
use pylon_token::gov_msg::{PollExecuteMsg, PollStatus, QueryMsg};
use pylon_token::gov_resp::{PollResponse, PollsResponse};

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

    let execute_data = vec![
        PollExecuteMsg {
            order: 1u64,
            contract: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::new(123),
            })
            .unwrap(),
        },
        PollExecuteMsg {
            order: 3u64,
            contract: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::new(1),
            })
            .unwrap(),
        },
        PollExecuteMsg {
            order: 2u64,
            contract: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::new(12),
            })
            .unwrap(),
        },
    ];

    let poll_11 = PollResponse {
        id: 1u64,
        creator: TEST_CREATOR.to_string(),
        status: PollStatus::InProgress,
        end_height: 20000u64,
        title: "test".to_string(),
        category: "test".to_string(),
        description: "test".to_string(),
        link: Some("http://google.com".to_string()),
        deposit_amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        execute_data: Some(execute_data.clone()),
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        staked_amount: None,
        total_balance_at_end_poll: None,
    };

    let poll_12 = PollResponse {
        id: 2u64,
        creator: TEST_CREATOR.to_string(),
        status: PollStatus::InProgress,
        end_height: 20000u64,
        title: "test".to_string(),
        category: "test2".to_string(),
        description: "test".to_string(),
        link: Some("http://google.com".to_string()),
        deposit_amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        execute_data: Some(execute_data),
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        staked_amount: None,
        total_balance_at_end_poll: None,
    };

    let poll_21 = PollResponse {
        id: 3u64,
        creator: TEST_CREATOR.to_string(),
        status: PollStatus::InProgress,
        end_height: 20000u64,
        title: "test2".to_string(),
        category: "test".to_string(),
        description: "test2".to_string(),
        link: None,
        deposit_amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        execute_data: None,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        staked_amount: None,
        total_balance_at_end_poll: None,
    };

    let poll_22 = PollResponse {
        id: 4u64,
        creator: TEST_CREATOR.to_string(),
        status: PollStatus::InProgress,
        end_height: 20000u64,
        title: "test2".to_string(),
        category: "test2".to_string(),
        description: "test2".to_string(),
        link: None,
        deposit_amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        execute_data: None,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        staked_amount: None,
        total_balance_at_end_poll: None,
    };

    vec![
        poll_11.clone(),
        poll_12.clone(),
        poll_21.clone(),
        poll_22.clone(),
    ]
    .iter()
    .for_each(|resp| {
        let _execute_res = contract::execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            create_poll_msg(
                Some(resp.title.clone()),
                Some(resp.category.clone()),
                Some(resp.description.clone()),
                resp.link.clone(),
                resp.execute_data.clone(),
            ),
        )
        .unwrap();
    });

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            category_filter: None,
            status_filter: None,
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
            poll_11.clone(),
            poll_12.clone(),
            poll_21.clone(),
            poll_22.clone()
        ]
    );

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            category_filter: None,
            status_filter: None,
            start_after: Some(2u64),
            limit: None,
            order_by: Some(OrderBy::Asc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls, vec![poll_21.clone(), poll_22.clone()]);

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            category_filter: None,
            status_filter: None,
            start_after: Some(3u64),
            limit: None,
            order_by: Some(OrderBy::Desc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls, vec![poll_12, poll_11.clone()]);

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            category_filter: Some("test".to_string()),
            status_filter: Some(PollStatus::InProgress),
            start_after: None,
            limit: None,
            order_by: Some(OrderBy::Asc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls, vec![poll_11.clone(), poll_21.clone()]);

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            category_filter: Some("test".to_string()),
            status_filter: Some(PollStatus::InProgress),
            start_after: None,
            limit: None,
            order_by: Some(OrderBy::Desc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls, vec![poll_21.clone(), poll_11]);

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            category_filter: None,
            status_filter: Some(PollStatus::InProgress),
            start_after: Some(2u64),
            limit: None,
            order_by: Some(OrderBy::Asc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls, vec![poll_21, poll_22]);

    let res = contract::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            category_filter: None,
            status_filter: Some(PollStatus::Passed),
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
