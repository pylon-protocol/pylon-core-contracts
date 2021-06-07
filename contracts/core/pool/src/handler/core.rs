use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    from_binary, log, to_binary, Api, BankMsg, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::{Cw20HandleMsg, Cw20ReceiveMsg};
use moneymarket::querier::deduct_tax;
use pylon_core::pool_msg::Cw20HookMsg;
use std::ops::Add;

use crate::config;
use crate::querier;

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender.clone();
    if let Some(msg) = cw20_msg.msg {
        match from_binary(&msg)? {
            Cw20HookMsg::Redeem {} => {
                // only asset contract can execute this message
                let config: config::Config = config::read(&deps.storage)?;
                if deps.api.canonical_address(&sender)? != config.dp_token {
                    return Err(StdError::unauthorized());
                }

                redeem(deps, env, cw20_msg.sender, cw20_msg.amount)
            }
        }
    } else {
        Err(StdError::generic_err(
            "Invalid request: \"redeem\" message not included in request",
        ))
    }
}

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config = config::read(&deps.storage)?;

    // check deposit
    let received: Uint256 = env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == config.stable_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    if received.is_zero() {
        return Err(StdError::generic_err(format!(
            "Pool: insufficient token amount {}",
            config.stable_denom,
        )));
    }

    let deposit_amount = deduct_tax(
        deps,
        Coin {
            denom: config.stable_denom.clone(),
            amount: received.into(),
        },
    )?
    .amount;

    Ok(HandleResponse {
        messages: [
            querier::feeder::update_msg(deps, &config.exchange_rate_feeder, &config.dp_token)?,
            querier::anchor::deposit_stable_msg(
                deps,
                &config.moneymarket,
                &config.stable_denom,
                deposit_amount,
            )?,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.dp_token)?,
                msg: to_binary(&Cw20HandleMsg::Mint {
                    recipient: env.message.sender.clone(),
                    amount: deposit_amount,
                })?,
                send: vec![],
            })],
        ]
        .concat(),
        log: vec![
            log("action", "deposit"),
            log("sender", env.message.sender),
            log("amount", deposit_amount.to_string()),
        ],
        data: None,
    })
}

pub fn redeem<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config = config::read(&deps.storage)?;

    let (market_redeem_amount, pool_redeem_amount, _) =
        querier::pool::calculate_return_amount(deps, &config, Uint256::from(amount))?;

    Ok(HandleResponse {
        messages: [
            querier::feeder::update_msg(deps, &config.exchange_rate_feeder, &config.dp_token)?,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.dp_token)?,
                msg: to_binary(&Cw20HandleMsg::Burn { amount })?,
                send: vec![],
            })],
            querier::anchor::redeem_stable_msg(
                deps,
                &config.moneymarket,
                &config.atoken,
                market_redeem_amount.into(),
            )?,
            vec![CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address,
                to_address: sender,
                amount: vec![pool_redeem_amount.clone()],
            })],
        ]
        .concat(),
        log: vec![
            log("action", "redeem"),
            log("sender", env.message.sender),
            log("amount", pool_redeem_amount.amount),
        ],
        data: None,
    })
}

pub fn claim_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // calculate (total_aust_amount * exchange_rate) - (total_dp_balance)
    let config = config::read(&deps.storage)?;
    if config.beneficiary != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::unauthorized());
    }

    let (reward_amount, fee_amount) =
        querier::pool::calculate_reward_amount(deps, &config, Option::from(env.block.time))?;
    let (market_redeem_amount, _, _) =
        querier::pool::calculate_return_amount(deps, &config, reward_amount.add(fee_amount))?;
    let (_, beneficiary_redeem_amount, _) =
        querier::pool::calculate_return_amount(deps, &config, reward_amount)?;
    let (_, collector_redeem_amount, _) =
        querier::pool::calculate_return_amount(deps, &config, fee_amount)?;

    Ok(HandleResponse {
        messages: [
            querier::feeder::update_msg(deps, &config.exchange_rate_feeder, &config.dp_token)?,
            querier::anchor::redeem_stable_msg(
                deps,
                &config.moneymarket,
                &config.atoken,
                market_redeem_amount.into(),
            )?,
            vec![
                CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: env.message.sender.clone(),
                    amount: vec![beneficiary_redeem_amount.clone()],
                }),
                CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: deps.api.human_address(&config.fee_collector)?,
                    amount: vec![collector_redeem_amount.clone()],
                }),
            ],
        ]
        .concat(),
        log: vec![
            log("action", "claim_reward"),
            log("sender", env.message.sender),
            log("amount", beneficiary_redeem_amount.amount),
            log("fee", collector_redeem_amount.amount),
        ],
        data: None,
    })
}

pub fn register_dp_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let mut config = config::read(&deps.storage)?;
    if config.dp_token != CanonicalAddr::default() {
        return Err(StdError::unauthorized());
    }

    config.dp_token = deps.api.canonical_address(&env.message.sender)?;
    config::store(&mut deps.storage, &config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("dp_token", env.message.sender)],
        data: None,
    })
}
