use cosmwasm_std::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Sub;

use crate::validator::Validator;

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
                return self.finish < env.block.time.seconds();
            }
            if self.finish == 0 {
                return env.block.time.seconds() < self.start;
            }
            env.block.time.seconds() < self.start || self.finish < env.block.time.seconds()
        } else {
            if self.start == 0 {
                return env.block.time.seconds() < self.finish;
            }
            if self.finish == 0 {
                return self.start < env.block.time.seconds();
            }
            self.start < env.block.time.seconds() && env.block.time.seconds() < self.finish
        }
    }

    pub fn configure(&mut self, start: Option<u64>, finish: Option<u64>) -> Vec<Attribute> {
        let mut attrs = vec![];
        if let Some(start) = start {
            self.start = start;
            attrs.push(Attribute::new("new_start_time", start.to_string()));
        }
        if let Some(finish) = finish {
            self.finish = finish;
            attrs.push(Attribute::new("new_finish_time", finish.to_string()));
        }
        attrs
    }
}
