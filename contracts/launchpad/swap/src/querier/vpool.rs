use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{StdError, StdResult};
use std::ops::{Div, Mul, Sub};

// x => UST amount
// y => MINE amount
// dx => UST amount to withdraw
// return => UST amount to receive
pub fn calculate_withdraw_amount(x: &Uint256, y: &Uint256, dx: &Uint256) -> StdResult<Uint256> {
    if x.lt(&dx) {
        return Err(StdError::generic_err("VPool: insufficient UST amount"));
    }
    Ok(x.mul(*y).div(Decimal256::from_uint256(x.sub(*dx))))
}

pub fn calculate_current_price(x: &Uint256, y: &Uint256) -> StdResult<Decimal256> {
    let liq_x = Decimal256::from_uint256(*x);
    let liq_y = Decimal256::from_uint256(*y);

    Ok(liq_x.div(liq_y))
}
