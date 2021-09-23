use cosmwasm_std::{log, Env, LogAttribute, StdError, StdResult};
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
            if self.start != 0 && self.finish != 0 {
                write!(f, "(~ {}, {} ~)", self.start, self.finish)
            } else if self.start == 0 {
                write!(f, "{} ~)", self.finish)
            } else {
                write!(f, "(~ {})", self.start)
            }
        }
        /* not inverse */
        else if self.start != 0 && self.finish != 0 {
            write!(f, "({} ~ {})", self.start, self.finish)
        } else if self.start == 0 {
            write!(f, "(~ {})", self.finish)
        } else {
            write!(f, "({} ~)", self.start)
        }
    }
}

impl Validator for TimeRange {
    fn validate(&self) -> StdResult<()> {
        if (self.start != 0 && self.finish != 0) && self.start.gt(&self.finish) {
            return Err(StdError::generic_err(
                "Gateway/Pool: time range validation failed. reason: finish < start",
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
            if self.start == 0 {
                return self.finish < env.block.time;
            }
            if self.finish == 0 {
                return env.block.time < self.start;
            }
            env.block.time < self.start || self.finish < env.block.time
        } else {
            if self.start == 0 {
                return env.block.time < self.finish;
            }
            if self.finish == 0 {
                return self.start < env.block.time;
            }
            self.start < env.block.time && env.block.time < self.finish
        }
    }

    pub fn configure(&mut self, start: Option<u64>, finish: Option<u64>) -> Vec<LogAttribute> {
        let mut logs = vec![];
        if let Some(start) = start {
            self.start = start;
            logs.push(log("new_start_time", start));
        }
        if let Some(finish) = finish {
            self.finish = finish;
            logs.push(log("new_finish_time", finish));
        }
        logs
    }
}
