use cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage};

use crate::config;
use crate::querier;
use crate::resp;

pub fn deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&resp::DepositAmountResponse {
        amount: querier::token::balance_of(deps, &config.dp_token, owner)?,
    })?)
}

pub fn total_deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&resp::TotalDepositAmountResponse {
        amount: querier::token::total_supply(deps, &config.dp_token)?,
    })?)
}

pub fn beneficiary<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&resp::BeneficiaryResponse {
        beneficiary: deps.api.human_address(&config.beneficiary)?,
    })?)
}

pub fn money_market<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&resp::MoneyMarketResponse {
        moneymarket: deps.api.human_address(&config.moneymarket)?,
    })?)
}

pub fn stable_denom<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&resp::StableDenomResponse {
        stable_denom: config.stable_denom.clone(),
    })?)
}

pub fn anchor_token<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&resp::ATokenResponse {
        anchor_token: deps.api.human_address(&config.atoken)?,
    })?)
}

pub fn dp_token<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&resp::DPTokenResponse {
        dp_token: deps.api.human_address(&config.dp_token)?,
    })?)
}

pub fn claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let (reward_amount, _) = querier::pool::calculate_reward_amount(deps, None)?;

    Ok(to_binary(&resp::ClaimableRewardResponse {
        claimable_reward: reward_amount.into(),
    })?)
}
