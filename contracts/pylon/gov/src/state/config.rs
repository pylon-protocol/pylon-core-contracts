use cosmwasm_std::{CanonicalAddr, Decimal, StdError, StdResult, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub pylon_token: CanonicalAddr,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub expiration_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, KEY_CONFIG)
}

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

impl Config {
    pub fn validate(&self) -> StdResult<()> {
        Config::validate_quorum(self.quorum)?;
        Config::validate_threshold(self.threshold)?;
        Ok(())
    }

    /// validate_quorum returns an error if the quorum is invalid
    /// (we require 0-1)
    pub fn validate_quorum(quorum: Decimal) -> StdResult<()> {
        if quorum > Decimal::one() {
            Err(StdError::generic_err("quorum must be 0 to 1"))
        } else {
            Ok(())
        }
    }

    /// validate_threshold returns an error if the threshold is invalid
    /// (we require 0-1)
    pub fn validate_threshold(threshold: Decimal) -> StdResult<()> {
        if threshold > Decimal::one() {
            Err(StdError::generic_err("threshold must be 0 to 1"))
        } else {
            Ok(())
        }
    }
}
