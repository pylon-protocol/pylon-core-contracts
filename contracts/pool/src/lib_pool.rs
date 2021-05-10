use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Api, Coin, Extern, Querier, StdResult, Storage};

use moneymarket::querier::deduct_tax;

use crate::config;
use crate::lib_anchor as anchor;
use crate::lib_token as token;

pub fn calculate_return_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amount: Uint256,
) -> StdResult<(Uint256, Coin, Coin)> {
    let config: config::Config = config::read(&deps.storage)?;
    let epoch_state = anchor::epoch_state(deps, &config.moneymarket)?;

    let market_redeem_amount = amount / epoch_state.exchange_rate; // calculate
    let pool_redeem_amount = deduct_tax(
        deps,
        Coin {
            denom: config.stable_denom.clone(),
            amount: market_redeem_amount.into(),
        },
    )?;
    let return_amount = deduct_tax(deps, pool_redeem_amount.clone())?;

    Ok((market_redeem_amount, pool_redeem_amount, return_amount))
}

pub fn calculate_reward_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Uint256> {
    let config: config::Config = config::read(&deps.storage)?;

    let epoch_state = anchor::epoch_state(deps, &config.moneymarket)?;
    let dp_total_supply = token::total_supply(deps, &config.dp_token)?;
    let atoken_balance =
        token::balance_of(deps, &config.atoken, deps.api.human_address(&config.this)?)?;

    let reward_amount = (epoch_state.exchange_rate * Uint256::from(atoken_balance))
        - Uint256::from(dp_total_supply);

    Ok(reward_amount)
}
