use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::{Env, MessageInfo, Uint128};

use crate::executions::poll::execute;
use crate::executions::ExecuteResult;
use crate::state::poll::VoteOption;
use crate::testing::{mock_deps, MockDeps, TEST_VOTER, VOTING_TOKEN};

#[allow(dead_code)]
pub fn exec(deps: &mut MockDeps, env: Env, _info: MessageInfo, poll_id: u64) -> ExecuteResult {
    execute(deps.as_mut(), env, poll_id)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_init_msg = super::instantiate::default_msg();

    const STAKE_AMOUNT: u128 = 1000;
    const POLL_ID: u64 = 1;
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

    super::poll_end::default(&mut deps, end_height, POLL_ID);
}

// use crate::entrypoints;
// use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
// use cosmwasm_std::{
//     attr, coins, from_binary, to_binary, Addr, CosmosMsg, SubMsg, Uint128, WasmMsg,
// };
// use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
// use pylon_token::common::OrderBy;
// use pylon_token::gov_msg::{
//     Cw20HookMsg, ExecuteMsg, PollExecuteMsg, PollMsg, PollStatus, QueryMsg, VoteOption,
// };
// use pylon_token::gov_resp::{PollResponse, PollsResponse};
//
// use crate::testing::assert::{assert_create_poll_result, assert_stake_tokens_result};
// use crate::testing::constants::*;
// use crate::testing::message::create_poll_msg;
// use crate::testing::mock_querier::mock_dependencies;
// use crate::testing::utils::{mock_env_height, mock_instantiate};
//
// #[test]
// fn add_several_execute_msgs() {
//     let mut deps = mock_dependencies(&[]);
//     mock_instantiate(deps.as_mut());
//
//     let info = mock_info(VOTING_TOKEN, &[]);
//     let env = mock_env_height(0, 10000);
//
//     let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
//         amount: Uint128::new(123),
//     })
//         .unwrap();
//
//     let exec_msg_bz2 = to_binary(&Cw20ExecuteMsg::Burn {
//         amount: Uint128::new(12),
//     })
//         .unwrap();
//
//     let exec_msg_bz3 = to_binary(&Cw20ExecuteMsg::Burn {
//         amount: Uint128::new(1),
//     })
//         .unwrap();
//
//     // push two execute msgs to the list
//     let execute_msgs: Vec<PollExecuteMsg> = vec![
//         PollExecuteMsg {
//             order: 1u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz,
//         },
//         PollExecuteMsg {
//             order: 3u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz3,
//         },
//         PollExecuteMsg {
//             order: 2u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz2,
//         },
//     ];
//
//     let msg = create_poll_msg(None, None, None, None, Some(execute_msgs.clone()));
//
//     let execute_res = entrypoints::execute(deps.as_mut(), env.clone(), info, msg).unwrap();
//     assert_create_poll_result(
//         1,
//         env.block.height + DEFAULT_VOTING_PERIOD,
//         TEST_CREATOR,
//         execute_res,
//         deps.as_ref(),
//     );
//
//     let res = entrypoints::query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
//     let value: PollResponse = from_binary(&res).unwrap();
//
//     let response_execute_data = value.execute_data.unwrap();
//     assert_eq!(response_execute_data.len(), 3);
//     assert_eq!(response_execute_data, execute_msgs);
// }
//
// #[test]
// fn execute_poll_with_order() {
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
//         amount: Uint128::new(10),
//     })
//         .unwrap();
//
//     let exec_msg_bz2 = to_binary(&Cw20ExecuteMsg::Burn {
//         amount: Uint128::new(20),
//     })
//         .unwrap();
//
//     let exec_msg_bz3 = to_binary(&Cw20ExecuteMsg::Burn {
//         amount: Uint128::new(30),
//     })
//         .unwrap();
//     let exec_msg_bz4 = to_binary(&Cw20ExecuteMsg::Burn {
//         amount: Uint128::new(40),
//     })
//         .unwrap();
//     let exec_msg_bz5 = to_binary(&Cw20ExecuteMsg::Burn {
//         amount: Uint128::new(50),
//     })
//         .unwrap();
//
//     //add three messages with different order
//     let execute_msgs: Vec<PollExecuteMsg> = vec![
//         PollExecuteMsg {
//             order: 3u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz3.clone(),
//         },
//         PollExecuteMsg {
//             order: 4u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz4.clone(),
//         },
//         PollExecuteMsg {
//             order: 2u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz2.clone(),
//         },
//         PollExecuteMsg {
//             order: 5u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz5.clone(),
//         },
//         PollExecuteMsg {
//             order: 1u64,
//             contract: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz.clone(),
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
//         .unwrap();
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
//         vote: VoteOption::Yes,
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
//     creator_info.sender = Addr::unchecked(TEST_CREATOR);
//     creator_env.block.height += DEFAULT_VOTING_PERIOD;
//
//     let msg = ExecuteMsg::Poll(PollMsg::End { poll_id: 1 });
//     let execute_res = entrypoints::execute(
//         deps.as_mut(),
//         creator_env.clone(),
//         creator_info.clone(),
//         msg,
//     )
//         .unwrap();
//
//     assert_eq!(
//         execute_res.attributes,
//         vec![
//             attr("action", "end_poll"),
//             attr("poll_id", "1"),
//             attr("rejected_reason", ""),
//             attr("passed", "true"),
//         ]
//     );
//     assert_eq!(
//         execute_res.messages,
//         vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: VOTING_TOKEN.to_string(),
//             msg: to_binary(&Cw20ExecuteMsg::Transfer {
//                 recipient: TEST_CREATOR.to_string(),
//                 amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
//             })
//                 .unwrap(),
//             funds: vec![],
//         }))]
//     );
//
//     // End poll will withdraw deposit balance
//     deps.querier.with_token_balances(&[(
//         &VOTING_TOKEN.to_string(),
//         &[(
//             &MOCK_CONTRACT_ADDR.to_string(),
//             &Uint128::from(stake_amount as u128),
//         )],
//     )]);
//
//     creator_env.block.height += DEFAULT_TIMELOCK_PERIOD;
//     let msg = ExecuteMsg::Poll(PollMsg::Execute { poll_id: 1 });
//     let execute_res =
//         entrypoints::execute(deps.as_mut(), creator_env.clone(), creator_info, msg).unwrap();
//     assert_eq!(
//         execute_res.messages,
//         vec![SubMsg::reply_on_error(
//             CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: creator_env.contract.address.to_string(),
//                 msg: to_binary(&ExecuteMsg::Poll(PollMsg::ExecuteMsgs { poll_id: 1 })).unwrap(),
//                 funds: vec![],
//             }),
//             1
//         )]
//     );
//
//     let msg = ExecuteMsg::Poll(PollMsg::ExecuteMsgs { poll_id: 1 });
//     let contract_info = mock_info(MOCK_CONTRACT_ADDR, &[]);
//     let execute_res = entrypoints::execute(deps.as_mut(), creator_env, contract_info, msg).unwrap();
//     assert_eq!(
//         execute_res.messages,
//         vec![
//             SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: VOTING_TOKEN.to_string(),
//                 msg: exec_msg_bz,
//                 funds: vec![],
//             })),
//             SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: VOTING_TOKEN.to_string(),
//                 msg: exec_msg_bz2,
//                 funds: vec![],
//             })),
//             SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: VOTING_TOKEN.to_string(),
//                 msg: exec_msg_bz3,
//                 funds: vec![],
//             })),
//             SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: VOTING_TOKEN.to_string(),
//                 msg: exec_msg_bz4,
//                 funds: vec![],
//             })),
//             SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: VOTING_TOKEN.to_string(),
//                 msg: exec_msg_bz5,
//                 funds: vec![],
//             })),
//         ]
//     );
//     assert_eq!(
//         execute_res.attributes,
//         vec![attr("action", "execute_poll"), attr("poll_id", "1"),]
//     );
// }
//
// #[test]
// fn poll_with_empty_execute_data_marked_as_executed() {
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
//     let msg = create_poll_msg(None, None, None, None, Some(vec![]));
//     let execute_res = entrypoints::execute(
//         deps.as_mut(),
//         creator_env.clone(),
//         creator_info.clone(),
//         msg,
//     )
//         .unwrap();
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
//         vote: VoteOption::Yes,
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
//     creator_info.sender = Addr::unchecked(TEST_CREATOR);
//     creator_env.block.height += DEFAULT_VOTING_PERIOD;
//
//     let msg = ExecuteMsg::Poll(PollMsg::End { poll_id: 1 });
//     let execute_res = entrypoints::execute(
//         deps.as_mut(),
//         creator_env.clone(),
//         creator_info.clone(),
//         msg,
//     )
//         .unwrap();
//
//     assert_eq!(
//         execute_res.attributes,
//         vec![
//             attr("action", "end_poll"),
//             attr("poll_id", "1"),
//             attr("rejected_reason", ""),
//             attr("passed", "true"),
//         ]
//     );
//     assert_eq!(
//         execute_res.messages,
//         vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: VOTING_TOKEN.to_string(),
//             msg: to_binary(&Cw20ExecuteMsg::Transfer {
//                 recipient: TEST_CREATOR.to_string(),
//                 amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
//             })
//                 .unwrap(),
//             funds: vec![],
//         }))]
//     );
//
//     // End poll will withdraw deposit balance
//     deps.querier.with_token_balances(&[(
//         &VOTING_TOKEN.to_string(),
//         &[(
//             &MOCK_CONTRACT_ADDR.to_string(),
//             &Uint128::from(stake_amount as u128),
//         )],
//     )]);
//
//     creator_env.block.height += DEFAULT_TIMELOCK_PERIOD;
//     let msg = ExecuteMsg::Poll(PollMsg::Execute { poll_id: 1 });
//     let execute_res =
//         entrypoints::execute(deps.as_mut(), creator_env.clone(), creator_info, msg).unwrap();
//     assert_eq!(
//         execute_res.messages,
//         vec![SubMsg::reply_on_error(
//             CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: creator_env.contract.address.to_string(),
//                 msg: to_binary(&ExecuteMsg::Poll(PollMsg::ExecuteMsgs { poll_id: 1 })).unwrap(),
//                 funds: vec![],
//             }),
//             1
//         )]
//     );
//
//     // Executes since empty polls are allowed
//     let msg = ExecuteMsg::Poll(PollMsg::ExecuteMsgs { poll_id: 1 });
//     let contract_info = mock_info(MOCK_CONTRACT_ADDR, &[]);
//     let execute_res = entrypoints::execute(deps.as_mut(), creator_env, contract_info, msg).unwrap();
//     assert_eq!(execute_res.messages, vec![]);
//     assert_eq!(
//         execute_res.attributes,
//         vec![attr("action", "execute_poll"), attr("poll_id", "1")]
//     );
//
//     let res = entrypoints::query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
//     let poll_res: PollResponse = from_binary(&res).unwrap();
//     assert_eq!(poll_res.status, PollStatus::Executed);
//
//     let res = entrypoints::query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::PollsWithStatusFilter {
//             status_filter: Some(PollStatus::Executed),
//             start_after: None,
//             limit: None,
//             order_by: Some(OrderBy::Desc),
//         },
//     )
//         .unwrap();
//     let polls_res: PollsResponse = from_binary(&res).unwrap();
//     assert_eq!(polls_res.polls[0], poll_res);
// }
//
// #[test]
// fn poll_with_none_execute_data_marked_as_executed() {
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
//     let msg = create_poll_msg(None, None, None, None, None);
//     let execute_res = entrypoints::execute(
//         deps.as_mut(),
//         creator_env.clone(),
//         creator_info.clone(),
//         msg,
//     )
//         .unwrap();
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
//         vote: VoteOption::Yes,
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
//     creator_info.sender = Addr::unchecked(TEST_CREATOR);
//     creator_env.block.height += DEFAULT_VOTING_PERIOD;
//
//     let msg = ExecuteMsg::Poll(PollMsg::End { poll_id: 1 });
//     let execute_res = entrypoints::execute(
//         deps.as_mut(),
//         creator_env.clone(),
//         creator_info.clone(),
//         msg,
//     )
//         .unwrap();
//
//     assert_eq!(
//         execute_res.attributes,
//         vec![
//             attr("action", "end_poll"),
//             attr("poll_id", "1"),
//             attr("rejected_reason", ""),
//             attr("passed", "true"),
//         ]
//     );
//     assert_eq!(
//         execute_res.messages,
//         vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: VOTING_TOKEN.to_string(),
//             msg: to_binary(&Cw20ExecuteMsg::Transfer {
//                 recipient: TEST_CREATOR.to_string(),
//                 amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
//             })
//                 .unwrap(),
//             funds: vec![],
//         }))]
//     );
//
//     // End poll will withdraw deposit balance
//     deps.querier.with_token_balances(&[(
//         &VOTING_TOKEN.to_string(),
//         &[(
//             &MOCK_CONTRACT_ADDR.to_string(),
//             &Uint128::from(stake_amount as u128),
//         )],
//     )]);
//
//     creator_env.block.height += DEFAULT_TIMELOCK_PERIOD;
//     let msg = ExecuteMsg::Poll(PollMsg::Execute { poll_id: 1 });
//     let execute_res =
//         entrypoints::execute(deps.as_mut(), creator_env.clone(), creator_info, msg).unwrap();
//     assert_eq!(
//         execute_res.messages,
//         vec![SubMsg::reply_on_error(
//             CosmosMsg::Wasm(WasmMsg::Execute {
//                 contract_addr: creator_env.contract.address.to_string(),
//                 msg: to_binary(&ExecuteMsg::Poll(PollMsg::ExecuteMsgs { poll_id: 1 })).unwrap(),
//                 funds: vec![],
//             }),
//             1
//         )]
//     );
//
//     // Executes since empty polls are allowed
//     let msg = ExecuteMsg::Poll(PollMsg::ExecuteMsgs { poll_id: 1 });
//     let contract_info = mock_info(MOCK_CONTRACT_ADDR, &[]);
//     let execute_res = entrypoints::execute(deps.as_mut(), creator_env, contract_info, msg).unwrap();
//     assert_eq!(execute_res.messages, vec![]);
//     assert_eq!(
//         execute_res.attributes,
//         vec![attr("action", "execute_poll"), attr("poll_id", "1")]
//     );
//
//     let res = entrypoints::query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
//     let poll_res: PollResponse = from_binary(&res).unwrap();
//     assert_eq!(poll_res.status, PollStatus::Executed);
//
//     let res = entrypoints::query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::PollsWithStatusFilter {
//             status_filter: Some(PollStatus::Executed),
//             start_after: None,
//             limit: None,
//             order_by: Some(OrderBy::Desc),
//         },
//     )
//         .unwrap();
//     let polls_res: PollsResponse = from_binary(&res).unwrap();
//     assert_eq!(polls_res.polls[0], poll_res);
// }
