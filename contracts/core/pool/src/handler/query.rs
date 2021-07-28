use cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage};

use pylon_core::pool_resp as resp;

use crate::querier::{pool, token};
use crate::state::config;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = config::read(&deps.storage)?;

    to_binary(&resp::ConfigResponse {
        id: config.id.clone(),
        name: config.name.clone(),
        factory: deps.api.human_address(&config.factory)?,
        beneficiary: deps.api.human_address(&config.beneficiary)?,
        fee_collector: deps.api.human_address(&config.fee_collector)?,
        yield_adapter: deps.api.human_address(&config.yield_adapter)?,
        input_denom: config.input_denom,
        yield_token: deps.api.human_address(&config.yield_token)?,
        dp_token: deps.api.human_address(&config.dp_token)?,
    })
}

pub fn deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let config = config::read(&deps.storage)?;

    to_binary(&resp::DepositAmountResponse {
        amount: token::balance_of(deps, &config.dp_token, owner)?,
    })
}

pub fn total_deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let config = config::read(&deps.storage)?;

    to_binary(&resp::TotalDepositAmountResponse {
        amount: token::total_supply(deps, &config.dp_token)?,
    })
}

pub fn claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let reward = pool::claimable_rewards(deps)?;

    to_binary(&resp::ClaimableRewardResponse {
        amount: reward.amount.into(),
        fee: reward.fee.into(),
    })
}
