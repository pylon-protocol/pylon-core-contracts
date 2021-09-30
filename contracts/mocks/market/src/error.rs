use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Mocks/Market: {0}")]
    Std(#[from] StdError),

    #[error("Mocks/Market: {0}")]
    Overflow(#[from] OverflowError),

    #[error(
    "Mocks/Market: unauthorized (action: {action:?}, expected: {expected:?}, actual: {actual:?})"
    )]
    Unauthorized {
        action: String,
        expected: String,
        actual: String,
    },

    #[error("Mocks/Market: invalid reply ID (ID: {id:?}")]
    InvalidReplyId { id: u64 },

    #[error("Mocks/Market: unsupported receive message. (type: {typ:?})")]
    UnsupportedReceiveMsg { typ: String },

    #[error("Core/Pool: zero amount not allowed")]
    NotAllowZeroAmount {},
}
