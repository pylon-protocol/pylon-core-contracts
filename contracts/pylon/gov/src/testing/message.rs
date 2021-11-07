use cosmwasm_std::{to_binary, Uint128};
use cw20::Cw20ReceiveMsg;
use pylon_token::gov_msg::{Cw20HookMsg, ExecuteMsg, PollExecuteMsg};

use crate::testing::constants::*;

pub fn create_poll_msg(
    title: Option<String>,
    category: Option<String>,
    description: Option<String>,
    link: Option<String>,
    execute_msg: Option<Vec<PollExecuteMsg>>,
) -> ExecuteMsg {
    ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_CREATOR.to_string(),
        amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        msg: to_binary(&Cw20HookMsg::CreatePoll {
            title: title.unwrap_or_else(|| "test".to_string()),
            category: category.unwrap_or_else(|| "test".to_string()),
            description: description.unwrap_or_else(|| "test".to_string()),
            link,
            execute_msgs: execute_msg,
        })
        .unwrap(),
    })
}
