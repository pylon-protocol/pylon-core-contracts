use cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage};

use crate::querier::vpool::calculate_current_price;
use crate::state;
use cosmwasm_bignumber::Uint256;
use pylon_launchpad::swap_resp as resp;
use terraswap::querier::query_balance;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = state::read_config(&deps.storage)?;

    Ok(to_binary(&resp::ConfigResponse {
        owner: config.owner,
        beneficiary: config.beneficiary,
        start: config.start,
        finish: config.finish,
    })?)
}

pub fn balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let user = state::read_user(&deps.storage, &deps.api.canonical_address(&owner)?)?;

    Ok(to_binary(&resp::BalanceOfResponse {
        balance: user.amount,
    })?)
}

pub fn total_supply<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = state::read_config(&deps.storage)?;
    let vpool = state::read_vpool(&deps.storage)?;

    let balance = query_balance(deps, &config.this, vpool.x_denom)?;

    Ok(to_binary(&resp::TotalSupplyResponse {
        total_supply: Uint256::from(balance),
    })?)
}

pub fn current_price<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let vpool = state::read_vpool(&deps.storage)?;

    Ok(to_binary(&resp::CurrentPriceResponse {
        price: calculate_current_price(&vpool.liq_x, &vpool.liq_y)?,
    })?)
}
