use cosmwasm_std::{Env, MessageInfo};

use crate::executions::poll::fail;
use crate::executions::ExecuteResult;
use crate::testing::MockDeps;

#[allow(dead_code)]
pub fn exec(deps: &mut MockDeps, _env: Env, _info: MessageInfo, poll_id: u64) -> ExecuteResult {
    fail(deps.as_mut(), poll_id)
}

// use crate::entrypoints;
// use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
// use cosmwasm_std::{
//     attr, coins, from_binary, to_binary, ContractResult, CosmosMsg, Reply, SubMsg, Uint128, WasmMsg,
// };
// use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
// use pylon_token::common::OrderBy;
// use pylon_token::gov_msg::{
//     Cw20HookMsg, ExecuteMsg, PollExecuteMsg, PollMsg, PollStatus, QueryMsg, VoteOption,
// };
// use pylon_token::gov_resp::{PollResponse, PollsResponse};
//
// use crate::error::ContractError;
// use crate::testing::assert::{assert_create_poll_result, assert_stake_tokens_result};
// use crate::testing::constants::*;
// use crate::testing::message::create_poll_msg;
// use crate::testing::mock_querier::mock_dependencies;
// use crate::testing::utils::{mock_env_height, mock_instantiate};
//
// #[test]
// fn fail_poll() {
//     const POLL_START_HEIGHT: u64 = 1000;
//     const POLL_ID: u64 = 1;
//     let stake_amount = 1000;
//
//     let mut deps = mock_dependencies(&coins(1000, VOTING_TOKEN));
//     mock_instantiate(deps.as_mut());
//
//     let mut creator_env = mock_env_height(POLL_START_HEIGHT, 10000);
//     let creator_info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));
//
//     let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
//         amount: Uint128::new(123),
//     })
//         .unwrap();
//     let execute_msgs: Vec<PollExecuteMsg> = vec![PollExecuteMsg {
//         order: 1u64,
//         contract: VOTING_TOKEN.to_string(),
//         msg: exec_msg_bz.clone(),
//     }];
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
//     // Execute Poll should send submsg ExecuteMsgs
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
//     // ExecuteMsgs should send poll messages
//     let msg = ExecuteMsg::Poll(PollMsg::ExecuteMsgs { poll_id: 1 });
//     let contract_info = mock_info(MOCK_CONTRACT_ADDR, &[]);
//     let execute_res = entrypoints::execute(deps.as_mut(), creator_env, contract_info, msg).unwrap();
//     assert_eq!(
//         execute_res.messages,
//         vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: VOTING_TOKEN.to_string(),
//             msg: exec_msg_bz,
//             funds: vec![],
//         }))]
//     );
//
//     // invalid reply id
//     let reply_msg = Reply {
//         id: 2,
//         result: ContractResult::Err("Error".to_string()),
//     };
//     let res = entrypoints::reply(deps.as_mut(), mock_env(), reply_msg);
//     assert_eq!(res, Err(ContractError::InvalidReplyId {}));
//
//     // correct reply id
//     let reply_msg = Reply {
//         id: 1,
//         result: ContractResult::Err("Error".to_string()),
//     };
//     let res = entrypoints::reply(deps.as_mut(), mock_env(), reply_msg).unwrap();
//     assert_eq!(
//         res.attributes,
//         vec![attr("action", "fail_poll"), attr("poll_id", "1")]
//     );
//
//     let res = entrypoints::query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
//     let poll_res: PollResponse = from_binary(&res).unwrap();
//     assert_eq!(poll_res.status, PollStatus::Failed);
//
//     let res = entrypoints::query(
//         deps.as_ref(),
//         mock_env(),
//         QueryMsg::PollsWithStatusFilter {
//             status_filter: Some(PollStatus::Failed),
//             start_after: None,
//             limit: None,
//             order_by: Some(OrderBy::Desc),
//         },
//     )
//         .unwrap();
//     let polls_res: PollsResponse = from_binary(&res).unwrap();
//     assert_eq!(polls_res.polls[0], poll_res);
// }
