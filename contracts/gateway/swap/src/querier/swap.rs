use cosmwasm_bignumber::Uint256;
use cosmwasm_std::StdResult;
use std::cmp::min;
use std::ops::{Add, Mul, Sub};

use crate::state::config;

pub fn calculate_additional_cap(config: &config::Config, staked: Uint256) -> StdResult<Uint256> {
    if config.min_stake_amount.gt(&staked) {
        return Ok(Uint256::zero());
    }

    let x = if config.max_stake_amount.is_zero() {
        staked
    } else {
        min(staked, config.max_stake_amount)
    };

    Ok(x.sub(config.min_stake_amount)
        .mul(config.additional_cap_per_token))
}

pub fn calculate_user_cap(config: &config::Config, staked: Uint256) -> StdResult<Uint256> {
    let additional_cap = calculate_additional_cap(config, staked).unwrap();

    Ok(if config.max_user_cap.is_zero() {
        config.min_user_cap.add(additional_cap)
    } else {
        min(config.min_user_cap.add(additional_cap), config.max_user_cap)
    })
}
