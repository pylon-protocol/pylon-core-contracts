use cosmwasm_std::{Env, HumanAddr, StdError, StdResult};

pub mod configure;
pub mod core;
pub mod migrate;
pub mod query;
pub mod router;
mod util_staking;

pub fn validate_sender(env: &Env, expected: &HumanAddr, method: &str) -> StdResult<()> {
    if !env.message.sender.eq(expected) {
        return Err(StdError::generic_err(format!(
            "Lockup: sender validation failed. method: {}, reason: {} !== {}",
            method, env.message.sender, expected
        )));
    }

    Ok(())
}
