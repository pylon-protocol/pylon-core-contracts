use cosmwasm_std::{Env, MessageInfo, Uint128};

use crate::executions::airdrop::deallocate;
use crate::executions::ExecuteResult;
use crate::testing::MockDeps;

#[allow(dead_code)]
pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    airdrop_id: u64,
    recipient: String,
    deallocate_amount: Uint128,
) -> ExecuteResult {
    deallocate(
        deps.as_mut(),
        env,
        info,
        airdrop_id,
        recipient,
        deallocate_amount,
    )
}
