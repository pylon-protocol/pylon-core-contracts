use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{to_binary, Api, Binary, Coin, Extern, HumanAddr, Querier, StdResult, Storage};
use pylon_gateway::swap_resp as resp;
use pylon_utils::tax::deduct_tax;

use crate::querier::vpool::{calculate_current_price, calculate_withdraw_amount};
use crate::state;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = state::read_config(&deps.storage)?;

    to_binary(&resp::ConfigResponse {
        owner: config.owner,
        beneficiary: config.beneficiary,
        start: config.start,
        finish: config.finish,
        price: config.price,
        total_sale_amount: config.total_sale_amount,
    })
}

pub fn balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let user = state::read_user(&deps.storage, &deps.api.canonical_address(&owner)?)?;

    to_binary(&resp::BalanceOfResponse {
        amount: user.amount,
    })
}

pub fn total_supply<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let reward = state::read_reward(&deps.storage)?;

    to_binary(&resp::TotalSupplyResponse {
        amount: reward.total_supply,
    })
}

pub fn current_price<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let vpool = state::read_vpool(&deps.storage)?;

    to_binary(&resp::CurrentPriceResponse {
        price: calculate_current_price(&vpool.liq_x, &vpool.liq_y)?,
    })
}

pub fn simulate_withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amount: Uint256,
) -> StdResult<Binary> {
    let vpool = state::read_vpool(&deps.storage)?;

    to_binary(&resp::SimulateWithdrawResponse {
        amount: Uint256::from(
            deduct_tax(
                deps,
                Coin {
                    denom: "uusd".parse().unwrap(),
                    amount: calculate_withdraw_amount(&vpool.liq_x, &vpool.liq_y, &amount)?.into(),
                },
            )?
            .amount,
        ),
    })
}
