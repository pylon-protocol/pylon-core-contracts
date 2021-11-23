use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error(
        "Core/Pool: Unauthorized (action: {action:?}, expected: {expected:?}, actual: {actual:?})"
    )]
    Unauthorized {
        action: String,
        expected: String,
        actual: String,
    },

    #[error("Core/Pool: Invalid reply ID (ID: {id:?}")]
    InvalidReplyId { id: u64 },

    #[error("Core/Pool: Zero amount not allowed")]
    NotAllowZeroAmount {},

    #[error("Core/Pool: other denom except {denom:?} is not allowed")]
    NotAllowOtherDenoms { denom: String },

    #[error("Core/Pool: other action except {action:?} is not allowed")]
    NotAllowOtherCw20ReceiveAction { action: String },
}
