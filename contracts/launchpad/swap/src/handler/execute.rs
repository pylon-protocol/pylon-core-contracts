use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    log, to_binary, Api, BankMsg, Coin, CosmosMsg, Env, Extern, HandleResponse, Querier, StdError,
    StdResult, Storage, WasmMsg,
};
use cw20::Cw20HandleMsg;
use std::ops::{Add, Sub};
use terraswap::querier::query_balance;

use crate::querier::tax::deduct_tax;
use crate::querier::vpool::calculate_withdraw_amount;
use crate::state;

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let vpool = state::read_vpool(&deps.storage)?;

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

    let deposit_amount = deduct_tax(
        deps,
        Coin {
            denom: vpool.x_denom,
            amount: received.into(),
        },
    )?
    .amount;

    let sender = &deps.api.canonical_address(&env.message.sender)?;
    let mut user = state::read_user(&deps.storage, sender)?;

    user.amount = user.amount.add(Uint256::from(deposit_amount));

    state::store_user(&mut deps.storage, sender, &user)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "deposit"),
            log("sender", env.message.sender),
            log("amount", deposit_amount),
        ],
        data: None,
    })
}

fn withdraw_with_penalty<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    let mut vpool = state::read_vpool(&deps.storage)?;
    let withdraw_amount = calculate_withdraw_amount(&vpool.liq_x, &vpool.liq_y, &amount)?;
    let penalty = amount.sub(withdraw_amount);

    vpool.liq_x = vpool.liq_x.sub(withdraw_amount);
    vpool.liq_y = vpool.liq_y.add(amount);

    state::store_vpool(&mut deps.storage, &vpool)?;

    let config = state::read_config(&deps.storage)?;

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
    let vpool = state::read_vpool(&deps.storage)?;

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
    let mut user = state::read_user(&deps.storage, sender)?;

    if user.amount.lt(&amount) {
        return Err(StdError::generic_err(format!(
            "Swap: insufficient amount to withdraw {} tokens",
            amount,
        )));
    }

    user.amount = user.amount.sub(amount);

    state::store_user(&mut deps.storage, sender, &user)?;

    let config: state::Config = state::read_config(&deps.storage)?;
    if config.finish.gt(&env.block.time) {
        Ok(withdraw_with_penalty(deps, env, amount)?)
    } else {
        Ok(withdraw_without_penalty(deps, env, amount)?)
    }
}

fn check_owner<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, env: &Env) -> StdResult<()> {
    let config = state::read_config(&deps.storage)?;
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

    let config = state::read_config(&deps.storage)?;
    if config.finish.gt(&env.block.time) {
        return Err(StdError::unauthorized());
    }

    let vpool = state::read_vpool(&deps.storage)?;
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
