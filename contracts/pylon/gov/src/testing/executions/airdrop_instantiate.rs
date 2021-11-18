use cosmwasm_std::{Env, MessageInfo, Uint128};

use crate::executions::airdrop::instantiate;
use crate::executions::ExecuteResult;
use crate::testing::MockDeps;

#[allow(dead_code)]
pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    start: u64,
    period: u64,
    reward_token: String,
    reward_amount: Uint128,
) -> ExecuteResult {
    instantiate(
        deps.as_mut(),
        env,
        info,
        start,
        period,
        reward_token,
        reward_amount,
    )
}
