use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::*;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use pylon_core::pool_msg::Cw20HookMsg;
use pylon_utils::tax::deduct_tax;
use pylon_utils::token;
use std::ops::{Div, Mul, Sub};
use std::str::FromStr;

use crate::config;
use crate::error::ContractError;
use crate::querier::anchor;

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Redeem {}) => {
            // only asset contract can execute this message
            let config: config::Config = config::read(deps.storage).unwrap();
            if deps.api.addr_canonicalize(info.sender.as_str()).unwrap() != config.dp_token {
                return Err(ContractError::Unauthorized {
                    action: "receive".to_string(),
                    expected: deps
                        .api
                        .addr_humanize(&config.dp_token)
                        .unwrap()
                        .to_string(),
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
        .find(|c| c.denom == config.stable_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);

    if received.is_zero() {
        return Err(ContractError::NotAllowZeroAmount {});
    }
    if info.funds.len() > 1 {
        return Err(ContractError::NotAllowOtherDenoms {
            denom: config.stable_denom,
        });
    }

    let dp_mint_amount = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: config.stable_denom.clone(),
            amount: received.into(),
        },
    )?
    .amount;

    Ok(Response::new()
        .add_messages(anchor::deposit_stable_msg(
            deps.as_ref(),
            &config.moneymarket,
            &config.stable_denom,
            received.into(),
        )?)
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: info.sender.to_string(),
                amount: dp_mint_amount,
            })?,
            funds: vec![],
        }))
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("amount", dp_mint_amount.to_string()))
}

pub fn redeem(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).unwrap();

    let epoch_state = anchor::epoch_state(deps.as_ref(), &config.moneymarket)?;
    let market_redeem_amount = Uint256::from(amount).div(epoch_state.exchange_rate);
    let user_redeem_amount = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: config.stable_denom.clone(),
            amount: deduct_tax(
                deps.as_ref(),
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

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn { amount })?,
            funds: vec![],
        }))
        .add_messages(anchor::redeem_stable_msg(
            deps.as_ref(),
            &config.moneymarket,
            &config.atoken,
            market_redeem_amount.into(),
        )?)
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.clone(),
            amount: vec![coin(
                u128::from(user_redeem_amount.amount),
                user_redeem_amount.denom.clone(),
            )],
        }))
        .add_attribute("action", "redeem")
        .add_attribute("sender", sender)
        .add_attribute("amount", user_redeem_amount.to_string()))
}

pub fn earn(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // calculate deduct(total_aust_amount * exchange_rate) - (total_dp_balance)
    let config = config::read(deps.storage).unwrap();
    if config.beneficiary != deps.api.addr_canonicalize(info.sender.as_str()).unwrap() {
        return Err(ContractError::Unauthorized {
            action: "earn".to_string(),
            expected: deps
                .api
                .addr_humanize(&config.beneficiary)
                .unwrap()
                .to_string(),
            actual: info.sender.to_string(),
        });
    }

    // assets
    let epoch_state = anchor::epoch_state(deps.as_ref(), &config.moneymarket)?;
    let atoken_balance = token::balance_of(
        deps.as_ref(),
        deps.api.addr_humanize(&config.atoken).unwrap().to_string(),
        env.contract.address.to_string(),
    )?;
    let dp_total_supply = token::total_supply(
        deps.as_ref(),
        deps.api
            .addr_humanize(&config.dp_token)
            .unwrap()
            .to_string(),
    )?;

    let pool_value_locked = Uint256::from(
        deduct_tax(
            deps.as_ref(),
            Coin {
                denom: config.stable_denom.clone(),
                amount: (atoken_balance.mul(epoch_state.exchange_rate)).into(),
            },
        )?
        .amount,
    );
    let earnable = pool_value_locked.sub(dp_total_supply);
    let fee = earnable.div(Decimal256::from_str("5.0")?); // TODO: fix it (20%)

    Ok(Response::new()
        .add_messages(anchor::redeem_stable_msg(
            deps.as_ref(),
            &config.moneymarket,
            &config.atoken,
            earnable.div(epoch_state.exchange_rate).into(),
        )?)
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: deps
                .api
                .addr_humanize(&config.beneficiary)
                .unwrap()
                .to_string(),
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: config.stable_denom.clone(),
                    amount: earnable.sub(fee).into(),
                },
            )?],
        }))
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: deps
                .api
                .addr_humanize(&config.fee_collector)
                .unwrap()
                .to_string(),
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: config.stable_denom.clone(),
                    amount: fee.into(),
                },
            )?],
        }))
        .add_attribute("action", "claim_reward")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("amount", earnable.sub(fee).to_string())
        .add_attribute("fee", fee.to_string()))
}

pub fn configure(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    beneficiary: Option<String>,
    fee_collector: Option<String>,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    if config.owner != deps.api.addr_canonicalize(info.sender.as_str()).unwrap() {
        return Err(ContractError::Unauthorized {
            action: "configure".to_string(),
            expected: deps.api.addr_humanize(&config.owner).unwrap().to_string(),
            actual: info.sender.to_string(),
        });
    }

    if let Some(beneficiary) = beneficiary {
        config.beneficiary = deps.api.addr_canonicalize(beneficiary.as_str()).unwrap();
    }
    if let Some(fee_collector) = fee_collector {
        config.fee_collector = deps.api.addr_canonicalize(fee_collector.as_str()).unwrap();
    }
    config::store(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn register_dp_token(
    deps: DepsMut,
    _env: Env,
    address: Addr,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).unwrap();
    if config.dp_token != CanonicalAddr::from(vec![]) {
        return Err(ContractError::Unauthorized {
            action: "register_dp_token".to_string(),
            expected: "<empty>".to_string(),
            actual: deps
                .api
                .addr_humanize(&config.dp_token)
                .unwrap()
                .to_string(),
        });
    }

    config.dp_token = deps.api.addr_canonicalize(address.as_str()).unwrap();
    config::store(deps.storage, &config)?;

    Ok(Response::new().add_attribute("dp_token", address.to_string()))
}
