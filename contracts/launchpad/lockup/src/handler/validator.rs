use crate::state;
use cosmwasm_std::{Env, HumanAddr, StdError, StdResult};
use pylon_launchpad::lockup_msg::ConfigureMsg;

pub fn validate_sender(env: &Env, expected: &HumanAddr, method: &str) -> StdResult<()> {
    if !env.message.sender.eq(expected) {
        return Err(StdError::generic_err(format!(
            "Lockup: sender validation failed. method: {}, reason: {} !== {}",
            method, env.message.sender, expected
        )));
    }

    Ok(())
}

fn validate_time(
    time_a: &u64,
    time_a_name: &str,
    time_b: &u64,
    time_b_name: &str,
) -> StdResult<()> {
    if time_a.gt(time_b) {
        return Err(StdError::generic_err(format!(
            "Lockup: time validation failed. reason: {} < {}",
            time_b_name, time_a_name,
        )));
    }

    Ok(())
}

pub fn validate_config_message(env: &Env, msg: &ConfigureMsg) -> StdResult<()> {
    if let Some(start_time) = msg.start_time {
        validate_time(&env.block.time, "now", &start_time, "start_time")?;
    }
    if let Some(cliff_time) = msg.cliff_time {
        validate_time(&env.block.time, "now", &cliff_time, "cliff_time")?;
    }
    if let Some(finish_time) = msg.finish_time {
        validate_time(&env.block.time, "now", &finish_time, "finish_time")?;
    }
    if let Some(temp_withdraw_start_time) = msg.temp_withdraw_start_time {
        validate_time(
            &env.block.time,
            "now",
            &temp_withdraw_start_time,
            "temp_withdraw_start_time",
        )?;
    }
    if let Some(temp_withdraw_finish_time) = msg.temp_withdraw_finish_time {
        validate_time(
            &env.block.time,
            "now",
            &temp_withdraw_finish_time,
            "temp_withdraw_finish_time",
        )?;
    }

    Ok(())
}

pub fn validate_config(config: &state::Config) -> StdResult<()> {
    validate_time(
        &config.temp_withdraw_start_time,
        "temp_withdraw_start_time",
        &config.temp_withdraw_finish_time,
        "temp_withdraw_finish_time",
    )?;
    validate_time(
        &config.start_time,
        "start_time",
        &config.finish_time,
        "finish_time",
    )?;
    validate_time(
        &config.cliff_time,
        "cliff_time",
        &config.finish_time,
        "finish_time",
    )?;

    Ok(())
}
