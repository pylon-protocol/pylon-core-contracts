use cosmwasm_bignumber::Uint256;
use cosmwasm_std::*;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use pylon_core::pool_v2_msg::Cw20HookMsg;
use pylon_token::collector::ExecuteMsg as CollectorHandleMsg;
use pylon_utils::tax::deduct_tax;
use std::ops::Div;

use crate::error::ContractError;
use crate::querier::{adapter, factory, pool};
use crate::state::config;

pub fn register_dp_token(
    deps: DepsMut,
    _env: Env,
    address: Addr,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    if config.dp_token != *"" {
        return Err(ContractError::Unauthorized {
            action: "register_dp_token".to_string(),
            expected: "<empty>".to_string(),
            actual: config.dp_token,
        });
    }

    config.dp_token = address.to_string();
    config::store(deps.storage, &config).unwrap();

    Ok(Response::new().add_attribute("dp_token", address.to_string()))
}

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Redeem {}) => {
            // only asset contract can execute this message
            let config = config::read(deps.storage)?;
            if config.dp_token.ne(&info.sender.to_string()) {
                return Err(ContractError::Unauthorized {
                    action: "receive".to_string(),
                    expected: config.dp_token,
                    actual: info.sender.to_string(),
                });
            }

            redeem(deps, env, info, cw20_msg.sender, cw20_msg.amount)
        }
        _ => Err(ContractError::NotAllowOtherCw20ReceiveAction {
            action: "redeem".to_string(),
        }),
    }
}

pub fn deposit(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).unwrap();

    // check deposit
    let received: Uint256 = info
        .funds
        .iter()
        .find(|c| c.denom == config.input_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    if received.is_zero() {
        return Err(ContractError::NotAllowZeroAmount {});
    }
    if info.funds.len() > 1 {
        return Err(ContractError::NotAllowOtherDenoms {
            denom: config.input_denom,
        });
    }

    let return_amount = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: config.input_denom.clone(),
            amount: received.into(),
        },
    )
    .unwrap();

    Ok(Response::new()
        .add_messages(adapter::deposit(
            deps.as_ref(),
            config.yield_adapter,
            received,
        )?)
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.dp_token,
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: info.sender.to_string(),
                amount: return_amount.amount,
            })
            .unwrap(),
            funds: vec![],
        }))
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("deposit_amount", received.to_string())
        .add_attribute("mint_amount", return_amount.amount.to_string()))
}

pub fn redeem(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).unwrap();

    let exchange_rate = adapter::exchange_rate(
        deps.as_ref(),
        config.yield_adapter.clone(),
        config.input_denom.clone(),
    )
    .unwrap();
    let return_amount = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: config.input_denom.clone(),
            amount: deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: config.input_denom.clone(),
                    amount,
                },
            )
            .unwrap()
            .amount,
        },
    )
    .unwrap();

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.dp_token,
            msg: to_binary(&Cw20ExecuteMsg::Burn { amount }).unwrap(),
            funds: vec![],
        }))
        .add_messages(adapter::redeem(
            deps.as_ref(),
            config.yield_adapter,
            Uint256::from(amount).div(exchange_rate),
        )?)
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.clone(),
            amount: vec![coin(
                u128::from(return_amount.amount),
                return_amount.denom.clone(),
            )],
        }))
        .add_attribute("action", "redeem")
        .add_attribute("sender", sender)
        .add_attribute("burn_amount", amount.to_string())
        .add_attribute("redeem_amount", return_amount.amount.to_string()))
}

pub fn earn(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // calculate deduct(total_aust_amount * exchange_rate) - (total_dp_balance)
    let config = config::read(deps.storage).unwrap();
    if config.beneficiary.ne(&info.sender.to_string()) {
        return Err(ContractError::Unauthorized {
            action: "earn".to_string(),
            expected: config.beneficiary,
            actual: info.sender.to_string(),
        });
    }

    let adapter_config = adapter::config(deps.as_ref(), config.yield_adapter.clone())?;
    let factory_config = factory::config(deps.as_ref(), config.factory.clone())?;
    let reward = pool::claimable_rewards(deps.as_ref(), env)?;
    let exchange_rate = adapter::exchange_rate(
        deps.as_ref(),
        config.yield_adapter.clone(),
        config.input_denom.clone(),
    )?;

    Ok(Response::new()
        .add_messages(adapter::redeem(
            deps.as_ref(),
            config.yield_adapter,
            reward.total().div(exchange_rate),
        )?)
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.beneficiary,
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: config.input_denom.clone(),
                    amount: reward.amount.into(),
                },
            )?],
        }))
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: factory_config.fee_collector.clone(),
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: config.input_denom,
                    amount: reward.fee.into(),
                },
            )?],
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: factory_config.fee_collector,
            msg: to_binary(&CollectorHandleMsg::Sweep {
                denom: adapter_config.input_denom,
            })
            .unwrap(),
            funds: vec![],
        }))
        .add_attribute("action", "claim_reward")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("reward", reward.amount.to_string())
        .add_attribute("fee", reward.fee.to_string()))
}
