use cosmwasm_bignumber::Uint256;
use cosmwasm_std::*;
use cw20::Cw20HandleMsg;
use pylon_utils::tax::deduct_tax;
use std::ops::{Add, Div, Mul, Sub};
use terraswap::querier::query_balance;

use crate::error::ContractError;
use crate::querier::staking::staker;
use crate::querier::swap::calculate_user_cap;
use crate::querier::vpool::calculate_withdraw_amount;
use crate::state::{config, state, user, vpool};

pub fn configure(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    total_sale_amount: Uint256,
    min_user_cap: Uint256,
    max_user_cap: Uint256,
) -> Result<Response, ContractError> {
    config::store(deps.storage).update(|mut config| {
        config.total_sale_amount = total_sale_amount;
        config.min_user_capr = min_user_cap;
        config.max_user_cap = max_user_cap;
        Ok(config)
    })?;

    Ok(Response::default())
}

pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).load()?;
    let vpool = vpool::read(deps.storage).load()?;

    if config.start.gt(&env.block.time.seconds()) {
        return Err(ContractError::Std(StdError::generic_err(
            "Swap: not started",
        )));
    }
    if config.finish.lt(&env.block.time.seconds()) {
        return Err(ContractError::Std(StdError::generic_err("Swap: finished")));
    }

    // 1:1
    let received: Uint256 = env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == vpool.x_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);
    if received.is_zero() {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Swap: insufficient token amount {}",
            vpool.x_denom,
        ))));
    }
    if env.message.sent_funds.len() > 1 {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Swap: this contract only accepts {}",
            vpool.x_denom,
        ))));
    }

    let sender = &deps.api.canonical_address(&env.message.sender)?;
    let mut user = user::read(deps.storage, sender)?;
    let mut state = state::read(deps.storage).load()?;

    let deposit_amount = received.div(config.base_price);
    if deposit_amount.lt(&config.min_user_cap) {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Swap: min user cap not satisfied ({})",
            config.min_user_cap,
        ))));
    }

    let staker_info = staker(
        deps.as_ref(),
        config.staking_contract.to_string(),
        info.sender.to_string(),
    )
    .unwrap();
    let user_cap = calculate_user_cap(&config, Uint256::from(staker_info.balance)).unwrap();
    if user.amount.add(deposit_amount).gt(&user_cap) {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Swap: user cap exceeded ({})",
            user_cap,
        ))));
    }
    if state
        .total_supply
        .add(deposit_amount)
        .gt(&config.total_sale_amount)
    {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Swap: sale cap exceeded ({})",
            config.total_sale_amount
        ))));
    }

    user.amount = user.amount.add(deposit_amount);
    state.total_supply = state.total_supply.add(deposit_amount);

    user::store(deps.storage, sender, &user)?;
    state::store(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sedner", info.sender.to_string())
        .add_attribute("amount", received.to_string())
        .add_attribute("allocated", deposit_amount.to_string()))
}

fn withdraw_with_penalty(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let mut vpool = vpool::read(deps.storage).load()?;
    let withdraw_amount = calculate_withdraw_amount(&vpool.liq_x, &vpool.liq_y, &amount)?;

    vpool.liq_x = vpool.liq_x.sub(withdraw_amount);
    vpool.liq_y = vpool.liq_y.add(amount);
    vpool::store(deps.storage).save(&vpool)?;

    let config = config::read(deps.storage).load()?;
    let penalty = amount.mul(config.base_price).sub(withdraw_amount);

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: vpool.x_denom.clone(),
                    amount: withdraw_amount.into(),
                },
            )?],
        }))
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.beneficiary.to_string(),
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: vpool.x_denom,
                    amount: penalty.into(),
                },
            )?],
        }))
        .add_attribute("action", "withdraw")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("amount", withdraw_amount.to_string())
        .add_attribute("penalty", penalty.to_string()))
}

fn withdraw_without_penalty(
    deps: DepsMut,
    env: Env,
    amount: Uint256,
) -> Result<Response, ContractError> {
    let vpool = vpool::read(deps.storage)?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&vpool.y_addr)?,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: env.message.sender.clone(),
                amount: amount.into(),
            })?,
            funds: vec![],
        })],
        log: vec![
            log("action", "withdraw"),
            log("sender", env.message.sender),
            log("amount", amount),
        ],
        data: None,
    })
}

pub fn withdraw(deps: DepsMut, env: Env, amount: Uint256) -> Result<Response, ContractError> {
    // xyk
    let sender = &deps.api.canonical_address(&env.message.sender)?;
    let mut user = user::read(deps.storage, sender)?;
    let mut state = state::read(deps.storage)?;

    if user.amount.lt(&amount) {
        return Err(ContractError::Std(StdError::generic_err(format!(
            "Swap: insufficient amount to withdraw {} tokens",
            amount,
        ))));
    }

    user.amount = user.amount.sub(amount);
    state.total_supply = state.total_supply.sub(amount);

    user::store(deps.storage, sender, &user)?;
    state::store(deps.storage, &state)?;

    let config = config::read(deps.storage)?;
    if config.finish.gt(env.block.time.seconds()) {
        Ok(withdraw_with_penalty(deps, env, amount)?)
    } else {
        Ok(withdraw_without_penalty(deps, env, amount)?)
    }
}

fn check_owner(deps: Deps, env: &Env) -> Result<(), ContractError> {
    let config = config::read(deps.storage)?;
    if config.owner.ne(&env.message.sender) {
        return Err(ContractError::Std(StdError::unauthorized()));
    }
    Ok(())
}

pub fn earn(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    check_owner(deps, &env)?;

    let config = config::read(deps.storage)?;
    if config.finish.gt(env.block.time.seconds()) {
        return Err(ContractError::Std(StdError::generic_err(
            "Swap: not finished",
        )));
    }

    let vpool = vpool::read(deps.storage)?;
    let earn_amount = query_balance(deps, &env.contract.address, vpool.x_denom.clone())?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address,
            to_address: config.beneficiary,
            amount: vec![deduct_tax(
                deps,
                Coin {
                    denom: vpool.x_denom,
                    amount: earn_amount,
                },
            )?],
        })],
        log: vec![
            log("action", "earn"),
            log("sender", env.message.sender),
            log("amount", earn_amount),
        ],
        data: None,
    })
}
