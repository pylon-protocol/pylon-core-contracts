use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Coin, Deps, Env, StdResult};
use pylon_utils::tax::deduct_tax;
use pylon_utils::token;
use std::ops::{Add, Mul, Sub};

use crate::querier::{adapter, factory};
use crate::state::config;

#[derive(Clone, Default, Debug)]
pub struct Reward {
    pub amount: Uint256,
    pub fee: Uint256,
}

impl Reward {
    pub fn total(&self) -> Uint256 {
        self.amount.add(self.fee)
    }
}

pub fn claimable_rewards(deps: Deps, env: Env) -> StdResult<Reward> {
    let config = config::read(deps.storage)?;

    let exchange_rate =
        adapter::exchange_rate(deps, config.yield_adapter, config.input_denom.clone())?;
    let yield_token_balance =
        token::balance_of(deps, config.yield_token, env.contract.address.to_string())?;
    let dp_total_supply = token::total_supply(deps, config.dp_token)?;

    let pvl = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.input_denom,
                amount: (yield_token_balance.mul(exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let amount = pvl.sub(dp_total_supply);
    let factory_config = factory::config(deps, config.factory)?;
    let fee = amount.mul(factory_config.fee_rate);

    Ok(Reward {
        amount: amount.sub(fee),
        fee,
    })
}
