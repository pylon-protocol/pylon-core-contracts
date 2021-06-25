use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{StdError, StdResult};
use std::ops::{Add, Div, Mul, Sub};

// x => UST amount
// y => MINE amount
// dx => UST amount to withdraw
// return => UST amount to receive
pub fn calculate_withdraw_amount(x: &Uint256, y: &Uint256, dy: &Uint256) -> StdResult<Uint256> {
    if x.lt(&dx) {
        return Err(StdError::generic_err("VPool: insufficient UST amount"));
    }
    let k = x.mul(*y);
    let dx = k.div(Decimal256::from_uint256(y.add(*dy)));
    Ok(k.div(Decimal256::from_uint256(x.add(dx))))
}

pub fn calculate_current_price(x: &Uint256, y: &Uint256) -> StdResult<Decimal256> {
    let liq_x = Decimal256::from_uint256(*x);
    let liq_y = Decimal256::from_uint256(*y);

    Ok(liq_x.div(liq_y))
}
