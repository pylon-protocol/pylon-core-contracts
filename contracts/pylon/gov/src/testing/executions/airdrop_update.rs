use cosmwasm_std::{Env, MessageInfo};

use crate::executions::airdrop::update;
use crate::executions::ExecuteResult;
use crate::testing::MockDeps;

#[allow(dead_code)]
pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    target: Option<String>,
) -> ExecuteResult {
    update(deps.as_mut(), env, info, target)
}
