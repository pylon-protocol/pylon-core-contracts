use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized (action: {action:?}, expected: {expected:?}, actual: {actual:?})")]
    Unauthorized {
        action: String,
        expected: String,
        actual: String,
    },
}
