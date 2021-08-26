use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    log, to_binary, Api, BankMsg, Coin, CosmosMsg, Env, Extern, HandleResponse, Querier, StdError,
    StdResult, Storage, WasmMsg,
};
use cw20::Cw20HandleMsg;
use pylon_utils::tax::deduct_tax;
use std::ops::{Add, Div, Mul, Sub};
use terraswap::querier::query_balance;

use crate::querier::staking::staker;
use crate::querier::swap::calculate_user_cap;
use crate::querier::vpool::calculate_withdraw_amount;
use crate::state::{config, state, user, vpool};

pub fn configure<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    total_sale_amount: Uint256,
    min_user_cap: Uint256,
    max_user_cap: Uint256,
) -> StdResult<HandleResponse> {
    let mut config = config::read(&deps.storage)?;

    config.total_sale_amount = total_sale_amount;
    config.min_user_cap = min_user_cap;
    config.max_user_cap = max_user_cap;

    config::store(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
}

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let config = config::read(&deps.storage)?;
    let vpool = vpool::read(&deps.storage)?;

    if config.start.gt(&env.block.time) {
        return Err(StdError::generic_err("Swap: not started"));
    }
    if config.finish.lt(&env.block.time) {
        return Err(StdError::generic_err("Swap: finished"));
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
        return Err(StdError::generic_err(format!(
            "Swap: insufficient token amount {}",
            vpool.x_denom,
        )));
    }
    if env.message.sent_funds.len() > 1 {
        return Err(StdError::generic_err(format!(
            "Swap: this contract only accepts {}",
            vpool.x_denom,
        )));
    }

    let sender = &deps.api.canonical_address(&env.message.sender)?;
    let mut user = user::read(&deps.storage, sender)?;
    let mut state = state::read(&deps.storage)?;

    let deposit_amount = received.div(config.base_price);
    if deposit_amount.lt(&config.min_user_cap) {
        return Err(StdError::generic_err(format!(
            "Swap: min user cap not satisfied ({})",
            config.min_user_cap,
        )));
    }

    let staker_info = staker(deps, &config.staking_contract, env.message.sender.clone()).unwrap();
    let user_cap = calculate_user_cap(&config, Uint256::from(staker_info.balance)).unwrap();
    if user.amount.add(deposit_amount).gt(&user_cap) {
        return Err(StdError::generic_err(format!(
            "Swap: user cap exceeded ({})",
            user_cap,
        )));
    }
    if state
        .total_supply
        .add(deposit_amount)
        .gt(&config.total_sale_amount)
    {
        return Err(StdError::generic_err(format!(
            "Swap: sale cap exceeded ({})",
            config.total_sale_amount
        )));
    }

    user.amount = user.amount.add(deposit_amount);
    state.total_supply = state.total_supply.add(deposit_amount);

    user::store(&mut deps.storage, sender, &user)?;
    state::store(&mut deps.storage, &state)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "deposit"),
            log("sender", env.message.sender),
            log("amount", received),
            log("allocated", deposit_amount),
        ],
        data: None,
    })
}

fn withdraw_with_penalty<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    let mut vpool = vpool::read(&deps.storage)?;
    let withdraw_amount = calculate_withdraw_amount(&vpool.liq_x, &vpool.liq_y, &amount)?;

    vpool.liq_x = vpool.liq_x.sub(withdraw_amount);
    vpool.liq_y = vpool.liq_y.add(amount);

    vpool::store(&mut deps.storage, &vpool)?;

    let config = config::read(&deps.storage)?;
    let penalty = amount.mul(config.base_price).sub(withdraw_amount);

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: env.message.sender.clone(),
                amount: vec![deduct_tax(
                    deps,
                    Coin {
                        denom: vpool.x_denom.clone(),
                        amount: withdraw_amount.into(),
                    },
                )?],
            }),
            CosmosMsg::Bank(BankMsg::Send {
                from_address: env.contract.address.clone(),
                to_address: config.beneficiary,
                amount: vec![deduct_tax(
                    deps,
                    Coin {
                        denom: vpool.x_denom,
                        amount: penalty.into(),
                    },
                )?],
            }),
        ],
        log: vec![
            log("action", "withdraw"),
            log("sender", env.message.sender),
            log("amount", withdraw_amount),
            log("penalty", penalty),
        ],
        data: None,
    })
}

fn withdraw_without_penalty<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    let vpool = vpool::read(&deps.storage)?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.human_address(&vpool.y_addr)?,
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: env.message.sender.clone(),
                amount: amount.into(),
            })?,
            send: vec![],
        })],
        log: vec![
            log("action", "withdraw"),
            log("sender", env.message.sender),
            log("amount", amount),
        ],
        data: None,
    })
}

pub fn withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    // xyk
    let sender = &deps.api.canonical_address(&env.message.sender)?;
    let mut user = user::read(&deps.storage, sender)?;
    let mut state = state::read(&deps.storage)?;

    if user.amount.lt(&amount) {
        return Err(StdError::generic_err(format!(
            "Swap: insufficient amount to withdraw {} tokens",
            amount,
        )));
    }

    user.amount = user.amount.sub(amount);
    state.total_supply = state.total_supply.sub(amount);

    user::store(&mut deps.storage, sender, &user)?;
    state::store(&mut deps.storage, &state)?;

    let config = config::read(&deps.storage)?;
    if config.finish.gt(&env.block.time) {
        Ok(withdraw_with_penalty(deps, env, amount)?)
    } else {
        Ok(withdraw_without_penalty(deps, env, amount)?)
    }
}

fn check_owner<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, env: &Env) -> StdResult<()> {
    let config = config::read(&deps.storage)?;
    if config.owner.ne(&env.message.sender) {
        return Err(StdError::unauthorized());
    }
    Ok(())
}

pub fn earn<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    check_owner(deps, &env)?;

    let config = config::read(&deps.storage)?;
    if config.finish.gt(&env.block.time) {
        return Err(StdError::generic_err("Swap: not finished"));
    }

    let vpool = vpool::read(&deps.storage)?;
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
