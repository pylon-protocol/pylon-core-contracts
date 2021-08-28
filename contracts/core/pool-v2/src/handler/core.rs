use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    from_binary, log, to_binary, Api, BankMsg, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::{Cw20HandleMsg, Cw20ReceiveMsg};
use pylon_core::pool_v2_msg::Cw20HookMsg;
use pylon_utils::tax::deduct_tax;
use std::ops::Div;

use crate::querier::{adapter, factory, pool};
use crate::state::config;

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
    let config = config::read(&deps.storage).unwrap();

    // check deposit
    let received: Uint256 = env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == config.input_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    if received.is_zero() {
        return Err(StdError::generic_err(format!(
            "Pool: zero {} amount",
            config.input_denom,
        )));
    }
    if env.message.sent_funds.len() > 1 {
        return Err(StdError::generic_err(format!(
            "Pool: this contract only accepts {}",
            config.input_denom,
        )));
    }

    let return_amount = deduct_tax(
        deps,
        Coin {
            denom: config.input_denom.clone(),
            amount: received.into(),
        },
    )
    .unwrap();

    Ok(HandleResponse {
        messages: [
            adapter::deposit(deps, &config.yield_adapter, received.into()).unwrap(),
            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.dp_token).unwrap(),
                msg: to_binary(&Cw20HandleMsg::Mint {
                    recipient: env.message.sender.clone(),
                    amount: return_amount.amount.clone(),
                })
                .unwrap(),
                send: vec![],
            })],
        ]
        .concat(),
        log: vec![
            log("action", "deposit"),
            log("sender", env.message.sender),
            log("deposit_amount", received),
            log("mint_amount", return_amount.amount),
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

    let exchange_rate =
        adapter::exchange_rate(deps, &config.yield_adapter, &config.input_denom).unwrap();
    let return_amount = deduct_tax(
        deps,
        Coin {
            denom: config.input_denom.clone(),
            amount: deduct_tax(
                deps,
                Coin {
                    denom: config.input_denom.clone(),
                    amount: amount.into(),
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
            adapter::redeem(
                deps,
                &config.yield_adapter,
                Uint256::from(amount).div(exchange_rate).into(),
            )?,
            vec![CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address,
                to_address: sender,
                amount: vec![return_amount.clone()],
            })],
        ]
        .concat(),
        log: vec![
            log("action", "redeem"),
            log("sender", env.message.sender),
            log("burn_amount", amount),
            log("redeem_amount", return_amount.amount),
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

    let factory_config = factory::config(deps, &config.factory)?;
    let reward = pool::claimable_rewards(deps)?;
    let exchange_rate = adapter::exchange_rate(deps, &config.yield_adapter, &config.input_denom)?;

    Ok(HandleResponse {
        messages: [
            adapter::redeem(
                deps,
                &config.yield_adapter,
                reward.total().div(exchange_rate).into(),
            )?,
            vec![
                CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: deps.api.human_address(&config.beneficiary)?,
                    amount: vec![deduct_tax(
                        deps,
                        Coin {
                            denom: config.input_denom.clone(),
                            amount: reward.amount.into(),
                        },
                    )?],
                }),
                CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: factory_config.fee_collector,
                    amount: vec![deduct_tax(
                        deps,
                        Coin {
                            denom: config.input_denom.clone(),
                            amount: reward.fee.into(),
                        },
                    )?],
                }),
            ],
        ]
        .concat(),
        log: vec![
            log("action", "claim_reward"),
            log("sender", env.message.sender),
            log("reward", reward.amount),
            log("fee", reward.fee),
        ],
        data: None,
    })
}
