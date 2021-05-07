use cosmwasm_std::{
    log, to_binary, Api, Binary, CanonicalAddr, CosmosMsg, Env, Extern, HandleResponse,
    InitResponse, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};

use crate::config::{read_config, store_config, Config};
use crate::msg::HandleMsg;
use cosmwasm_bignumber::Uint256;
use cw20::{Cw20CoinHuman, MinterResponse};

pub fn handle_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {
    // fetch ust

    let config: Config = read_config(&deps.storage)?;

    // Check base denom deposit
    let deposit_amount: Uint256 = _env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    if deposit_amount.is_zero() {
        return Err(StdError::generic_err(format!(
            "Pool: insufficient token amount {}",
            config.stable_denom,
        )));
    }

    // deposit to strategy

    // mint dp tokens

    Ok(HandleResponse::default())
}

pub fn handle_redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    // fetch dp tokens & burn them all

    // withdraw from strategy

    Ok(HandleResponse::default())
}

pub fn handle_claim_reward<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {
    // calculate (total_aust_amount * exchange_rate) - (total_dp_balance)

    // transfer to beneficiary or send Distribute(amount: Uint128) message

    Ok(HandleResponse::default())
}

pub fn handle_register_dp_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
) -> StdResult<HandleResponse> {
    let mut config: Config = read_config(&deps.storage)?;
    if config.dp_token != CanonicalAddr::default() {
        return Err(StdError::unauthorized());
    }

    config.dp_token = deps.api.canonical_address(&_env.message.sender)?;
    store_config(&mut deps.storage, &config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("dp_token", _env.message.sender)],
        data: None,
    })
}
