use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    from_binary, log, to_binary, Api, BankMsg, CanonicalAddr, Coin, CosmosMsg, Env, Extern,
    HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use std::ops::Div;

use cw20::{Cw20HandleMsg, Cw20ReceiveMsg};
use moneymarket::querier::deduct_tax;
use pylon_core::adapter::{Cw20HookMsg as AdapterHookMsg, HandleMsg as AdapterHandleMsg};
use pylon_core::pool_msg::Cw20HookMsg;

use crate::querier::{adapter, pool};
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
    let config = config::read(&deps.storage)?;

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

    // pool => adapters => anchor ( deduct tax twice )
    let dp_mint_amount = deduct_tax(
        deps,
        deduct_tax(
            deps,
            Coin {
                denom: config.input_denom.clone(),
                amount: received.into(),
            },
        )?,
    )?
    .amount;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.yield_adapter)?,
                msg: to_binary(&AdapterHandleMsg::Deposit {})?,
                send: vec![deduct_tax(
                    deps,
                    Coin {
                        denom: config.input_denom,
                        amount: received.into(),
                    },
                )?],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.dp_token)?,
                msg: to_binary(&Cw20HandleMsg::Mint {
                    recipient: env.message.sender.clone(),
                    amount: dp_mint_amount.clone(),
                })?,
                send: vec![],
            }),
        ],
        log: vec![
            log("action", "deposit"),
            log("sender", env.message.sender),
            log("deposit_amount", received),
            log("mint_amount", dp_mint_amount),
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

    let exchange_rate = adapter::exchange_rate(deps, &config.yield_adapter, &config.input_denom)?;
    let market_redeem_amount = Uint256::from(amount).div(exchange_rate);
    let user_redeem_amount = deduct_tax(
        deps,
        deduct_tax(
            deps,
            Coin {
                denom: config.input_denom.clone(),
                amount: amount.into(),
            },
        )?,
    )?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.dp_token)?,
                msg: to_binary(&Cw20HandleMsg::Burn { amount })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.yield_token)?,
                msg: to_binary(&Cw20HandleMsg::Send {
                    contract: deps.api.human_address(&config.yield_adapter)?,
                    amount: market_redeem_amount.into(),
                    msg: Option::from(to_binary(&AdapterHookMsg::Redeem {})?),
                })?,
                send: vec![],
            }),
            CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address,
                to_address: sender,
                amount: vec![user_redeem_amount.clone()],
            }),
        ],
        log: vec![
            log("action", "redeem"),
            log("sender", env.message.sender),
            log("burn_amount", amount),
            log("redeem_amount", user_redeem_amount.amount),
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

    let reward = pool::claimable_rewards(deps)?;
    let exchange_rate = adapter::exchange_rate(deps, &config.yield_adapter, &config.input_denom)?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.yield_token)?,
                msg: to_binary(&Cw20HandleMsg::Send {
                    contract: deps.api.human_address(&config.yield_adapter)?,
                    amount: reward.total().div(exchange_rate).into(),
                    msg: Option::from(to_binary(&AdapterHookMsg::Redeem {})?),
                })?,
                send: vec![],
            }),
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
                to_address: deps.api.human_address(&config.fee_collector)?,
                amount: vec![deduct_tax(
                    deps,
                    Coin {
                        denom: config.input_denom.clone(),
                        amount: reward.fee.into(),
                    },
                )?],
            }),
        ],
        log: vec![
            log("action", "claim_reward"),
            log("sender", env.message.sender),
            log("reward", reward.amount),
            log("fee", reward.fee),
        ],
        data: None,
    })
}
