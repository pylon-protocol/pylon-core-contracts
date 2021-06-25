use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{StdError, StdResult};
use std::ops::{Add, Div, Mul};

// x => UST amount
// y => MINE amount
// dx => UST amount to withdraw
// return => UST amount to receive
pub fn calculate_withdraw_amount(x: &Uint256, y: &Uint256, dy: &Uint256) -> StdResult<Uint256> {
    let k = x.mul(*y);
    Ok(k.div(Decimal256::from_uint256(y.add(*dy))))
}

pub fn calculate_current_price(x: &Uint256, y: &Uint256) -> StdResult<Decimal256> {
    let liq_x = Decimal256::from_uint256(*x);
    let liq_y = Decimal256::from_uint256(*y);

    Ok(liq_x.div(liq_y))
}
