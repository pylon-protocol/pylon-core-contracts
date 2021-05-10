use cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage};

use crate::config;
use crate::lib_pool as pool;
use crate::lib_token as token;
use crate::resp::{
    ATokenResponse, BeneficiaryResponse, ClaimableRewardResponse, DPTokenResponse,
    DepositAmountResponse, MoneyMarketResponse, StableDenomResponse, TotalDepositAmountResponse,
};

pub fn deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&DepositAmountResponse {
        amount: token::balance_of(deps, &config.dp_token, owner)?,
    })?)
}

pub fn total_deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&TotalDepositAmountResponse {
        amount: token::total_supply(deps, &config.dp_token)?,
    })?)
}

pub fn beneficiary<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&BeneficiaryResponse {
        beneficiary: deps.api.human_address(&config.beneficiary)?,
    })?)
}

pub fn money_market<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&MoneyMarketResponse {
        moneymarket: deps.api.human_address(&config.moneymarket)?,
    })?)
}

pub fn stable_denom<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&StableDenomResponse {
        stable_denom: config.stable_denom.clone(),
    })?)
}

pub fn anchor_token<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&ATokenResponse {
        anchor_token: deps.api.human_address(&config.atoken)?,
    })?)
}

pub fn dp_token<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: config::Config = config::read(&deps.storage)?;

    Ok(to_binary(&DPTokenResponse {
        dp_token: deps.api.human_address(&config.dp_token)?,
    })?)
}

pub fn claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let reward_amount = pool::calculate_reward_amount(deps)?;

    Ok(to_binary(&ClaimableRewardResponse {
        claimable_reward: reward_amount.into(),
    })?)
}
