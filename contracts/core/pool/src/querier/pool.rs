use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Api, Coin, Extern, Querier, StdResult, Storage};
use std::ops::{Add, Div, Mul, Sub};

use moneymarket::querier::deduct_tax;

use crate::querier::{adapter, factory, token};
use crate::state::config;

pub struct Reward {
    pub amount: Uint256,
    pub fee: Uint256,
}

impl Reward {
    pub fn total(&self) -> Uint256 {
        return self.amount.add(self.fee);
    }
}

pub fn claimable_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Reward> {
    let config = config::read(&deps.storage)?;

    let exchange_rate = adapter::exchange_rate(deps, &config.yield_adapter, &config.input_denom)?;
    let yield_token_balance = token::balance_of(
        deps,
        &config.yield_token,
        deps.api.human_address(&config.this)?,
    )?;
    let dp_total_supply = token::total_supply(deps, &config.dp_token)?;

    let pvl = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.input_denom.clone(),
                amount: (Uint256::from(yield_token_balance).mul(exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let amount = pvl.sub(Uint256::from(dp_total_supply));
    let fee_rate = factory::fee_rate(deps, &config.factory, &config.yield_adapter)?;
    let fee = amount.div(fee_rate);

    Ok(Reward {
        amount: amount.sub(fee),
        fee,
    })
}