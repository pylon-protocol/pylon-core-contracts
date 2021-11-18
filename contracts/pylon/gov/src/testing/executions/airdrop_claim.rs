use cosmwasm_std::{Env, MessageInfo};

use crate::executions::airdrop::claim;
use crate::executions::ExecuteResult;
use crate::testing::MockDeps;

#[allow(dead_code)]
pub fn exec(deps: &mut MockDeps, env: Env, info: MessageInfo, sender: String) -> ExecuteResult {
    claim(deps.as_mut(), env, info, sender)
}
