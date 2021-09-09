use cosmwasm_std::{Env, StdError, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Sub;

use crate::state::Validator;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeRange {
    pub start: u64,
    pub finish: u64,
    pub inverse: bool,
}

impl Default for TimeRange {
    fn default() -> Self {
        TimeRange {
            start: 0,
            finish: 0,
            inverse: false,
        }
    }
}

impl Display for TimeRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.inverse {
            write!(f, "(~ {}, {} ~)", self.start, self.finish)
        } else {
            write!(f, "({} ~ {})", self.start, self.finish)
        }
    }
}

impl Validator for TimeRange {
    fn validate(&self) -> StdResult<()> {
        if self.start.gt(&self.finish) {
            return Err(StdError::generic_err(
                "Lockup: time range validation failed. reason: finish < start",
            ));
        }

        Ok(())
    }
}

impl TimeRange {
    pub fn period(&self) -> u64 {
        if self.inverse {
            0
        } else {
            self.finish.sub(self.start)
        }
    }

    pub fn is_in_range(&self, env: &Env) -> bool {
        if self.inverse {
            env.block.time < self.start && self.finish < env.block.time
        } else {
            self.start < env.block.time && env.block.time < self.finish
        }
    }
}
