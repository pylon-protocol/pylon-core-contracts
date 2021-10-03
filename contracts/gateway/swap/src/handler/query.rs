use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{to_binary, Binary, Coin, Deps, Env, StdResult};
use pylon_gateway::swap_resp as resp;
use pylon_utils::tax::deduct_tax;

use crate::querier::{strategy, vpool};
use crate::state::{config, state, user};

pub fn config(deps: Deps) -> StdResult<Binary> {
    let config = config::read(deps.storage).load().unwrap();

    to_binary(&resp::ConfigResponse {
        owner: config.owner,
        beneficiary: config.beneficiary,
        start: config.start,
        finish: config.finish,
        price: config.price,
        total_sale_amount: config.swap_pool_size,
    })
}

pub fn balance_of(deps: Deps, owner: String) -> StdResult<Binary> {
    let user = user::read(
        deps.storage,
        &deps.api.addr_canonicalize(owner.as_str()).unwrap(),
    )
    .unwrap();

    to_binary(&resp::BalanceOfResponse {
        amount: user.swapped_in,
    })
}

pub fn is_whitelisted(deps: Deps, address: String) -> StdResult<Binary> {
    let config = config::read(deps.storage).load().unwrap();
    let user = user::read(
        deps.storage,
        &deps.api.addr_canonicalize(address.as_str()).unwrap(),
    )
    .unwrap();

    to_binary(&resp::IsWhitelistedResponse {
        whitelisted: !config.whitelist_enabled || user.whitelisted,
    })
}

pub fn available_cap_of(deps: Deps, address: String) -> StdResult<Binary> {
    let config = config::read(deps.storage).load().unwrap();
    if let Some(strategy) = config.cap_strategy {
        let available_cap = strategy::available_cap_of(deps, strategy, address)?;
        to_binary(&resp::AvailableCapOfResponse {
            amount: Option::Some(available_cap),
            unlimited: false,
        })
    } else {
        to_binary(&resp::AvailableCapOfResponse {
            amount: None,
            unlimited: true,
        })
    }
}

pub fn claimable_token_of(deps: Deps, env: Env, address: String) -> StdResult<Binary> {
    let claimable_token = strategy::claimable_token_of(deps, env.block.time.seconds(), address)?;

    to_binary(&resp::ClaimableTokenOfResponse {
        amount: claimable_token,
    })
}

pub fn total_supply(deps: Deps) -> StdResult<Binary> {
    let state = state::read(deps.storage).load().unwrap();

    to_binary(&resp::TotalSupplyResponse {
        amount: state.total_swapped,
    })
}

pub fn current_price(deps: Deps) -> StdResult<Binary> {
    let state = state::read(deps.storage).load().unwrap();

    to_binary(&resp::CurrentPriceResponse {
        price: vpool::calculate_current_price(&state).unwrap(),
    })
}

pub fn simulate_withdraw(
    deps: Deps,
    address: Option<String>,
    amount: Uint256,
) -> StdResult<Binary> {
    let config = config::read(deps.storage).load().unwrap();
    let state = state::read(deps.storage).load().unwrap();
    let (withdraw_amount, penalty) = vpool::calculate_penalty(&state, config.price, &amount)?;

    let mut withdrawable = true;
    if let Some(address) = address {
        let user = user::read(
            deps.storage,
            &deps.api.addr_canonicalize(address.as_str()).unwrap(),
        )
        .unwrap();

        withdrawable = user.swapped_out_claimed.is_zero();
    }

    to_binary(&resp::SimulateWithdrawResponse {
        amount: Uint256::from(
            deduct_tax(
                deps,
                Coin {
                    denom: "uusd".to_string(),
                    amount: withdraw_amount.into(),
                },
            )
            .unwrap()
            .amount,
        ),
        penalty,
        withdrawable,
    })
}
