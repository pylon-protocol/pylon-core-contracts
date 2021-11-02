use cosmwasm_std::{to_binary, Uint128};
use cw20::Cw20ReceiveMsg;
use pylon_token::gov_msg::{Cw20HookMsg, ExecuteMsg, PollExecuteMsg};

use crate::testing::constants::*;

pub fn create_poll_msg(
    title: String,
    description: String,
    link: Option<String>,
    execute_msg: Option<Vec<PollExecuteMsg>>,
) -> ExecuteMsg {
    ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_CREATOR.to_string(),
        amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        msg: to_binary(&Cw20HookMsg::CreatePoll {
            title,
            description,
            link,
            execute_msgs: execute_msg,
        })
        .unwrap(),
    })
}
