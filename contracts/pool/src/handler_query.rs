use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Env, Extern, HumanAddr, Querier, QueryRequest, StdResult,
    Storage, Uint128, WasmQuery,
};

use crate::config::{read_config, Config};
use crate::lib_pool::calculate_reward_amount;
use crate::lib_token::{token_balance_of, token_total_supply};
use crate::resp::{
    GetATokenResponse, GetBeneficiaryResponse, GetClaimableRewardResponse, GetDPTokenResponse,
    GetDepositAmountResponse, GetMoneyMarketResponse, GetStableDenomResponse,
    GetTotalDepositAmountResponse,
};
use cw20::{BalanceResponse, TokenInfoResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub fn query_deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<GetDepositAmountResponse> {
    let config: Config = read_config(&deps.storage)?;

    Ok(GetDepositAmountResponse {
        amount: token_balance_of(deps, &config.dp_token, owner)?,
    })
}

pub fn query_total_deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetTotalDepositAmountResponse> {
    let config: Config = read_config(&deps.storage)?;

    Ok(GetTotalDepositAmountResponse {
        amount: token_total_supply(deps, &config.dp_token)?,
    })
}

pub fn query_beneficiary<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetBeneficiaryResponse> {
    let config: Config = read_config(&deps.storage)?;

    Ok(GetBeneficiaryResponse {
        beneficiary: deps.api.human_address(&config.beneficiary)?,
    })
}

pub fn query_money_market<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetMoneyMarketResponse> {
    let config: Config = read_config(&deps.storage)?;

    Ok(GetMoneyMarketResponse {
        moneymarket: deps.api.human_address(&config.moneymarket)?,
    })
}

pub fn query_stable_denom<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetStableDenomResponse> {
    let config: Config = read_config(&deps.storage)?;

    Ok(GetStableDenomResponse {
        stable_denom: config.stable_denom.clone(),
    })
}

pub fn query_anchor_token<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetATokenResponse> {
    let config: Config = read_config(&deps.storage)?;

    Ok(GetATokenResponse {
        anchor_token: deps.api.human_address(&config.atoken)?,
    })
}

pub fn query_dp_token<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetDPTokenResponse> {
    let config: Config = read_config(&deps.storage)?;

    Ok(GetDPTokenResponse {
        dp_token: deps.api.human_address(&config.dp_token)?,
    })
}

pub fn query_claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetClaimableRewardResponse> {
    let reward_amount = calculate_reward_amount(deps)?;

    Ok(GetClaimableRewardResponse {
        claimable_reward: reward_amount.into(),
    })
}
