use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error(
        "Gateway/Swap: Unauthorized (action: {action:?}, expected: {expected:?}, actual: {actual:?})"
    )]
    Unauthorized {
        action: String,
        expected: String,
        actual: String,
    },

    #[error(
        "Gateway/Swap: withdraw amount exceeds user swapped_in amount (Available: {available:?})"
    )]
    WithdrawAmountExceeded { available: Uint256 },

    #[error("Gateway/Swap: swapped_in amount exceeds available cap (Available: {available:?})")]
    AvailableCapExceeded { available: Uint256 },

    #[error("Gateway/Swap: swapped_out amount exceeds pool size (Available: {available:?})")]
    PoolSizeExceeded { available: Uint256 },

    #[error("Gateway/Swap: Invalid reply ID (ID: {id:?}")]
    InvalidReplyId { id: u64 },

    #[error("Gateway/Swap: Zero amount not allowed")]
    NotAllowZeroAmount {},

    #[error("Gateway/Swap: other denom except {denom:?} is not allowed")]
    NotAllowOtherDenoms { denom: String },

    #[error("Gateway/Swap: {address:?} is not whitelisted")]
    NotAllowNonWhitelisted { address: String },

    #[error("Gateway/Swap: refund not allowed after token claim")]
    NotAllowWithdrawAfterClaim {},
}
