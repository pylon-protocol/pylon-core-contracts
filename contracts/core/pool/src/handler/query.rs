use cosmwasm_bignumber::Uint256;
use cosmwasm_std::Coin;
use cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage};
use moneymarket::querier::deduct_tax;
use std::ops::{Mul, Sub};

use crate::config;
use crate::querier;
use pylon_core::pool_resp as resp;

pub fn deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    to_binary(&resp::DepositAmountResponse {
        amount: querier::token::balance_of(deps, &config.dp_token, owner)?,
    })
}

pub fn total_deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    to_binary(&resp::TotalDepositAmountResponse {
        amount: querier::token::total_supply(deps, &config.dp_token)?,
    })
}

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    to_binary(&resp::ConfigResponse {
        beneficiary: deps.api.human_address(&config.beneficiary)?,
        moneymarket: deps.api.human_address(&config.moneymarket)?,
        stable_denom: config.stable_denom,
        anchor_token: deps.api.human_address(&config.atoken)?,
        dp_token: deps.api.human_address(&config.dp_token)?,
    })
}

pub fn claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let config = config::read(&deps.storage)?;

    // assets
    let epoch_state = querier::anchor::epoch_state(deps, &config.moneymarket)?;
    let virtual_exchange_rate = querier::feeder::fetch(
        deps,
        &config.exchange_rate_feeder,
        Option::None,
        &deps.api.human_address(&config.dp_token)?,
    )?;

    // collector
    let atoken_balance =
        querier::token::balance_of(deps, &config.atoken, deps.api.human_address(&config.this)?)?;
    let dp_total_supply = querier::token::total_supply(deps, &config.dp_token)?;
    let pool_value_locked = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.stable_denom.clone(),
                amount: (Uint256::from(atoken_balance).mul(epoch_state.exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let vpool_value_locked = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.stable_denom.clone(),
                amount: (Uint256::from(atoken_balance).mul(virtual_exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let earnable = pool_value_locked.sub(Uint256::from(dp_total_supply));
    let fee = pool_value_locked.sub(vpool_value_locked);

    to_binary(&resp::ClaimableRewardResponse {
        amount: earnable.sub(fee).into(),
    })
}
