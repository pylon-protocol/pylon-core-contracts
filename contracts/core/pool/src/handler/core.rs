use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    from_binary, log, to_binary, Api, BankMsg, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::{Cw20HandleMsg, Cw20ReceiveMsg};
use moneymarket::querier::deduct_tax;
use pylon_core::pool_msg::Cw20HookMsg;
use std::ops::{Div, Mul, Sub};
use std::str::FromStr;

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
    if env.message.sent_funds.len() > 1 {
        return Err(StdError::generic_err(format!(
            "Pool: this contract only accepts {}",
            config.stable_denom,
        )));
    }

    let dp_mint_amount = deduct_tax(
        deps,
        Coin {
            denom: config.stable_denom.clone(),
            amount: received.into(),
        },
    )?
    .amount;

    Ok(HandleResponse {
        messages: [
            querier::anchor::deposit_stable_msg(
                deps,
                &config.moneymarket,
                &config.stable_denom,
                received.into(),
            )?,
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.dp_token)?,
                msg: to_binary(&Cw20HandleMsg::Mint {
                    recipient: env.message.sender.clone(),
                    amount: dp_mint_amount,
                })?,
                send: vec![],
            })],
        ]
        .concat(),
        log: vec![
            log("action", "deposit"),
            log("sender", env.message.sender),
            log("amount", dp_mint_amount),
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

    let epoch_state = querier::anchor::epoch_state(deps, &config.moneymarket)?;
    let market_redeem_amount = Uint256::from(amount).div(epoch_state.exchange_rate);
    let user_redeem_amount = deduct_tax(
        deps,
        Coin {
            denom: config.stable_denom.clone(),
            amount: deduct_tax(
                deps,
                Coin {
                    denom: config.stable_denom.clone(),
                    amount,
                },
            )
            .unwrap()
            .amount,
        },
    )
    .unwrap();

    Ok(HandleResponse {
        messages: [
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
                amount: vec![user_redeem_amount.clone()],
            })],
        ]
        .concat(),
        log: vec![
            log("action", "redeem"),
            log("sender", env.message.sender),
            log("amount", user_redeem_amount.amount),
        ],
        data: None,
    })
}

pub fn earn<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // calculate deduct(total_aust_amount * exchange_rate) - (total_dp_balance)
    let config = config::read(&deps.storage)?;
    if config.beneficiary != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::generic_err(format!(
            "Pool: cannot execute earn function with unauthorized sender. (sender: {})",
            env.message.sender,
        )));
    }

    // assets
    let epoch_state = querier::anchor::epoch_state(deps, &config.moneymarket)?;
    let atoken_balance =
        querier::token::balance_of(deps, &config.atoken, env.contract.address.clone())?;
    let dp_total_supply = querier::token::total_supply(deps, &config.dp_token)?;

    let pool_value_locked = Uint256::from(
        deduct_tax(
            deps,
            Coin {
                denom: config.stable_denom.clone(),
                amount: (Uint256::from(atoken_balance).mul(epoch_state.exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let earnable = pool_value_locked.sub(Uint256::from(dp_total_supply));
    let fee = earnable.div(Decimal256::from_str("5.0")?); // TODO: fix it (20%)

    Ok(HandleResponse {
        messages: [
            querier::anchor::redeem_stable_msg(
                deps,
                &config.moneymarket,
                &config.atoken,
                earnable.div(epoch_state.exchange_rate).into(),
            )?,
            vec![
                CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: deps.api.human_address(&config.beneficiary)?,
                    amount: vec![deduct_tax(
                        deps,
                        Coin {
                            denom: config.stable_denom.clone(),
                            amount: earnable.sub(fee).into(),
                        },
                    )?],
                }),
                CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: deps.api.human_address(&config.fee_collector)?,
                    amount: vec![deduct_tax(
                        deps,
                        Coin {
                            denom: config.stable_denom.clone(),
                            amount: fee.into(),
                        },
                    )?],
                }),
            ],
        ]
        .concat(),
        log: vec![
            log("action", "claim_reward"),
            log("sender", env.message.sender),
            log("amount", earnable.sub(fee)),
            log("fee", fee),
        ],
        data: None,
    })
}

pub fn configure<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    beneficiary: HumanAddr,
    fee_collector: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut config = config::read(&deps.storage)?;
    if config.owner != deps.api.canonical_address(&env.message.sender).unwrap() {
        return Err(StdError::generic_err(format!(
            "Pool: cannot execute configure function with unauthorized sender. (sender: {})",
            env.message.sender,
        )));
    }

    config.beneficiary = deps.api.canonical_address(&beneficiary)?;
    config.fee_collector = deps.api.canonical_address(&fee_collector)?;
    config::store(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
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
