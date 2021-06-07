use cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage};

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
    let (reward_amount, _) = querier::pool::calculate_reward_amount(deps, &config, None)?;

    to_binary(&resp::ClaimableRewardResponse {
        claimable_reward: reward_amount.into(),
    })
}
