use cosmwasm_std::{Env, MessageInfo, Uint128};

use crate::executions::airdrop::allocate;
use crate::executions::ExecuteResult;
use crate::testing::MockDeps;

#[allow(dead_code)]
pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    airdrop_id: u64,
    recipient: String,
    allocate_amount: Uint128,
) -> ExecuteResult {
    allocate(
        deps.as_mut(),
        env,
        info,
        airdrop_id,
        recipient,
        allocate_amount,
    )
}
