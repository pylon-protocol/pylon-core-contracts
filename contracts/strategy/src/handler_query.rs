use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage,
    Uint128, WasmQuery,
};
use cw20::Cw20QueryMsg::{Balance, TokenInfo};

use crate::config::{read_config, Config};
use crate::resp::{
    GetBeneficiaryResponse, GetClaimableRewardResponse, GetDepositAmountResponse,
    GetStrategyResponse, GetTotalDepositAmountResponse,
};
use cw20::{BalanceResponse, TokenInfoResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub fn query_deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<GetDepositAmountResponse> {
    let config: Config = read_config(&deps.storage)?;

    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps.api.human_address(&config.dp_token)?,
        msg: to_binary(&Balance { address: owner })?,
    }))?;

    Ok(GetDepositAmountResponse {
        amount: balance.balance,
    })
}

pub fn query_total_deposit_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetTotalDepositAmountResponse> {
    let config: Config = read_config(&deps.storage)?;

    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&config.dp_token)?,
            msg: to_binary(&TokenInfo {})?,
        }))?;

    Ok(GetTotalDepositAmountResponse {
        amount: token_info.total_supply,
    })
}

pub fn query_strategy<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetStrategyResponse> {
    let config: Config = read_config(&deps.storage)?;

    Ok(GetStrategyResponse {
        strategy: deps.api.human_address(&config.strategy)?,
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

pub fn query_claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<GetClaimableRewardResponse> {
    let config: Config = read_config(&deps.storage)?;

    // TODO: query to config.strategy

    Ok(GetClaimableRewardResponse {
        claimable_reward: Uint128::zero(), // TODO
    })
}
