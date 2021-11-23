use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::*;
use pylon_core::pool_resp as resp;
use pylon_utils::tax::deduct_tax;
use pylon_utils::token;
use std::ops::{Div, Mul, Sub};
use std::str::FromStr;

use crate::config;
use crate::querier::anchor;

pub fn deposit_amount(deps: Deps, _env: Env, owner: String) -> StdResult<Binary> {
    let config: config::Config = config::read(deps.storage).unwrap();

    to_binary(&resp::DepositAmountResponse {
        amount: token::balance_of(
            deps,
            deps.api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
            owner,
        )?,
    })
}

pub fn total_deposit_amount(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config: config::Config = config::read(deps.storage).unwrap();

    to_binary(&resp::TotalDepositAmountResponse {
        amount: token::total_supply(
            deps,
            deps.api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
        )?,
    })
}

pub fn config(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config: config::Config = config::read(deps.storage).unwrap();

    to_binary(&resp::ConfigResponse {
        beneficiary: deps
            .api
            .addr_humanize(&config.beneficiary)
            .unwrap()
            .to_string(),
        fee_collector: deps
            .api
            .addr_humanize(&config.fee_collector)
            .unwrap()
            .to_string(),
        moneymarket: deps
            .api
            .addr_humanize(&config.moneymarket)
            .unwrap()
            .to_string(),
        stable_denom: config.stable_denom,
        anchor_token: deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        dp_token: deps
            .api
            .addr_humanize(&config.dp_token)
            .unwrap()
            .to_string(),
    })
}

pub fn claimable_reward(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    // assets
    let epoch_state = anchor::epoch_state(deps, &config.moneymarket)?;
    let atoken_balance = token::balance_of(
        deps,
        deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        env.contract.address.to_string(),
    )?;
    let dp_total_supply = token::total_supply(
        deps,
        deps.api
            .addr_humanize(&config.dp_token)
            .unwrap()
            .to_string(),
    )?;

    let pool_value_locked = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.stable_denom,
                amount: (atoken_balance.mul(epoch_state.exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let earnable = pool_value_locked.sub(dp_total_supply);
    let fee = earnable.div(Decimal256::from_str("5.0")?); // TODO: fix it (20%)

    to_binary(&resp::ClaimableRewardResponse {
        amount: earnable.sub(fee),
        fee,
    })
}
