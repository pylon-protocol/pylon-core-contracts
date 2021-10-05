use crate::state::state;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::StdResult;
use std::ops::{Add, Div, Mul, Sub};

// x => UST amount
// y => MINE amount
// dx => UST amount to withdraw
// return => UST amount to receive
pub fn calculate_withdraw_amount(state: &state::State, dy: &Uint256) -> StdResult<Uint256> {
    let k = state.liq_x.mul(state.liq_y);
    let dx = state
        .liq_x
        .sub(k.div(Decimal256::from_uint256(state.liq_y.add(*dy))));
    Ok(dx)
}

pub fn calculate_current_price(state: &state::State) -> StdResult<Decimal256> {
    let liq_x = Decimal256::from_uint256(state.liq_x);
    let liq_y = Decimal256::from_uint256(state.liq_y);

    Ok(liq_x.div(liq_y))
}
