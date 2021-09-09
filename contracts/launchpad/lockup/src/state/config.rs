use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Env, HumanAddr, StdError, StdResult, Storage};
use cosmwasm_storage::{ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::ops::Mul;

use crate::state::time_range::TimeRange;
use crate::state::Validator;

pub static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositConfig {
    pub time: TimeRange,
    pub user_cap: Uint256,
    pub total_cap: Uint256,
}

impl Default for DepositConfig {
    fn default() -> Self {
        DepositConfig {
            time: TimeRange::default(),
            user_cap: Uint256::zero(),
            total_cap: Uint256::zero(),
        }
    }
}

impl Validator for DepositConfig {
    fn validate(&self) -> StdResult<()> {
        self.time.validate()?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TempDepositConfig {
    pub time: TimeRange,
    pub user_cap: Uint256,
}

impl Default for TempDepositConfig {
    fn default() -> Self {
        TempDepositConfig {
            time: TimeRange::default(),
            user_cap: Uint256::zero(),
        }
    }
}

impl Validator for TempDepositConfig {
    fn validate(&self) -> StdResult<()> {
        self.time.validate()?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DistributionConfig {
    pub time: TimeRange,
    pub reward_rate: Decimal256,
    pub total_reward_amount: Uint256,
}

impl Validator for DistributionConfig {
    fn validate(&self) -> StdResult<()> {
        self.time.validate()?;

        let calculated_total_rewards = Uint256::from(self.time.period()).mul(self.reward_rate);
        if calculated_total_rewards.ne(&self.total_reward_amount) {
            return Err(StdError::generic_err(format!(
                "Lockup: distribution config validation failed. reason: total reward mismatch, expected: {}, actual: {}",
                self.total_reward_amount, calculated_total_rewards
            )));
        }

        Ok(())
    }
}

impl DistributionConfig {
    pub fn applicable_finish_time(&self, env: &Env) -> u64 {
        min(self.time.finish, env.block.time)
    }

    pub fn applicable_start_time(&self, env: &Env) -> u64 {
        max(self.time.start, env.block.time)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: HumanAddr,
    // share
    pub share_token: HumanAddr,
    pub deposit_config: DepositConfig,
    pub withdraw_time: TimeRange,
    pub temp_deposit_config: TempDepositConfig,
    pub temp_withdraw_time: TimeRange,
    // reward
    pub reward_token: HumanAddr,
    pub claim_time: TimeRange,
    pub distribution_config: DistributionConfig,
}

impl Validator for Config {
    fn validate(&self) -> StdResult<()> {
        // share
        self.deposit_config.validate()?;
        self.withdraw_time.validate()?;
        self.temp_deposit_config.validate()?;
        self.temp_withdraw_time.validate()?;

        // reward
        self.claim_time.validate()?;
        self.distribution_config.validate()?;

        Ok(())
    }
}

fn generate_time_range_error(
    action: &str,
    origin: &TimeRange,
    temp: Option<&TimeRange>,
) -> StdResult<()> {
    if let Some(temp) = temp {
        Err(StdError::generic_err(format!(
            "Lockup: current blocktime does not satisfies configured {} time range. origin: {}, temp: {}",
            action, origin, temp,
        )))
    } else {
        Err(StdError::generic_err(format!(
            "Lockup: current blocktime does not satisfies configured {} time range. origin: {}",
            action, origin,
        )))
    }
}

fn check_time_range(
    env: &Env,
    origin: &TimeRange,
    temp: Option<&TimeRange>,
    action: &str,
) -> StdResult<()> {
    let mut is_in_time_range = origin.is_in_range(env);
    if let Some(temp) = temp {
        is_in_time_range = is_in_time_range || temp.is_in_range(env);
    }

    if !is_in_time_range {
        generate_time_range_error(action, origin, temp)?
    }

    Ok(())
}

impl Config {
    pub fn check_deposit_time(&self, env: &Env) -> StdResult<()> {
        check_time_range(
            env,
            &self.deposit_config.time,
            Option::from(&self.temp_deposit_config.time),
            "deposit",
        )
    }

    pub fn check_withdraw_time(&self, env: &Env) -> StdResult<()> {
        check_time_range(
            env,
            &self.withdraw_time,
            Option::from(&self.temp_withdraw_time),
            "withdraw",
        )
    }

    pub fn check_claim_time(&self, env: &Env) -> StdResult<()> {
        check_time_range(env, &self.claim_time, Option::None, "claim")
    }
}

pub fn store<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    data.validate()?;
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read<S: Storage>(storage: &S) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}
