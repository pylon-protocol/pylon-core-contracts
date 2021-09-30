use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{to_binary, Api, Binary, Coin, Deps, Querier, StdResult, Storage};
use pylon_gateway::swap_resp as resp;
use pylon_utils::tax::deduct_tax;
use std::ops::{Mul, Sub};

use crate::querier::staking::staker;
use crate::querier::swap::calculate_user_cap;
use crate::querier::vpool::{calculate_current_price, calculate_withdraw_amount};
use crate::state::{config, state, user, vpool};

pub fn config(deps: Deps) -> StdResult<Binary> {
    let config = config::read(deps.storage).load().unwrap();

    to_binary(&resp::ConfigResponse {
        owner: config.owner.to_string(),
        beneficiary: config.beneficiary.to_string(),
        start: config.start,
        finish: config.finish,
        price: config.base_price,
        total_sale_amount: config.total_sale_amount,
    })
}

pub fn balance_of(deps: Deps, owner: String) -> StdResult<Binary> {
    let user = user::read(
        deps.storage,
        &deps.api.addr_canonicalize(owner.as_str()).unwrap(),
    )
    .unwrap();

    to_binary(&resp::BalanceOfResponse {
        amount: user.amount,
    })
}

pub fn available_cap_of(deps: Deps, address: String) -> StdResult<Binary> {
    let config = config::read(deps.storage).load().unwrap();
    let user = user::read(
        deps.storage,
        &deps.api.addr_canonicalize(address.as_str()).unwrap(),
    )
    .unwrap();
    let staker_info = staker(deps, config.staking_contract.to_string(), address).unwrap();
    if Uint256::from(staker_info.balance).lt(&config.min_stake_amount) {
        return to_binary(&resp::AvailableCapOfResponse {
            staked: Uint256::from(staker_info.balance),
            cap: Uint256::zero(),
        });
    }
    let cap = calculate_user_cap(&config, Uint256::from(staker_info.balance)).unwrap();

    to_binary(&resp::AvailableCapOfResponse {
        staked: Uint256::from(staker_info.balance),
        cap: cap.sub(user.amount).mul(config.base_price),
    })
}

pub fn total_supply(deps: Deps) -> StdResult<Binary> {
    let state = state::read(deps.storage).load().unwrap();

    to_binary(&resp::TotalSupplyResponse {
        amount: state.total_supply,
    })
}

pub fn current_price(deps: Deps) -> StdResult<Binary> {
    let vpool = vpool::read(deps.storage).load().unwrap();

    to_binary(&resp::CurrentPriceResponse {
        price: calculate_current_price(&vpool.liq_x, &vpool.liq_y).unwrap(),
    })
}

pub fn simulate_withdraw(deps: Deps, amount: Uint256) -> StdResult<Binary> {
    let vpool = vpool::read(deps.storage).load().unwrap();

    to_binary(&resp::SimulateWithdrawResponse {
        amount: Uint256::from(
            deduct_tax(
                deps,
                Coin {
                    denom: "uusd".parse().unwrap(),
                    amount: calculate_withdraw_amount(&vpool.liq_x, &vpool.liq_y, &amount)
                        .unwrap()
                        .into(),
                },
            )
            .unwrap()
            .amount,
        ),
    })
}
