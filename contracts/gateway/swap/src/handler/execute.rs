use cosmwasm_bignumber::Uint256;
use cosmwasm_std::*;
use cw20::Cw20ExecuteMsg;
use pylon_gateway::cap_strategy_msg::QueryMsg as CapQueryMsg;
use pylon_gateway::cap_strategy_resp;
use pylon_gateway::swap_msg::Strategy;
use pylon_utils::tax::deduct_tax;

use crate::error::ContractError;
use crate::querier::strategy;
use crate::querier::vpool;
use crate::state::{config, state, user};

pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).load()?;
    let state = state::read(deps.storage).load()?;

    if env.block.time.seconds() < config.start {
        return Err(ContractError::SwapNotStarted {
            start: config.start,
        });
    }
    if config.finish < env.block.time.seconds() {
        return Err(ContractError::SwapFinished {
            finish: config.finish,
        });
    }

    // 1:1
    let swapped_in: Uint256 = info
        .funds
        .iter()
        .find(|c| c.denom == state.x_denom)
        .map(|c| Uint256::from(c.amount))
        .unwrap_or_else(Uint256::zero);
    if swapped_in.is_zero() {
        return Err(ContractError::NotAllowZeroAmount {});
    }
    if info.funds.len() > 1 {
        return Err(ContractError::NotAllowOtherDenoms {
            denom: state.x_denom,
        });
    }

    let sender = &deps.api.addr_canonicalize(info.sender.as_str())?;
    let mut user = user::read(deps.storage, sender)?;
    let mut state = state::read(deps.storage).load()?;

    // check whitelisted, or free to participate everyone
    if config.whitelist_enabled && !user.whitelisted {
        return Err(ContractError::NotAllowNonWhitelisted {
            address: info.sender.to_string(),
        });
    }

    // check available cap via calling cap_strategy contract
    if let Some(strategy) = config.cap_strategy {
        let resp: cap_strategy_resp::AvailableCapOfResponse = deps.querier.query_wasm_smart(
            strategy,
            &CapQueryMsg::AvailableCapOf {
                amount: user.swapped_in,
                address: info.sender.to_string(),
            },
        )?;

        if let Some(v) = resp.amount {
            if v < swapped_in {
                return Err(ContractError::AvailableCapExceeded { available: v });
            }
        }
    } // or remains cap strategy to unlimited

    // check swap pool size
    let swapped_out = swapped_in / config.price;
    if state.total_swapped + swapped_out > config.swap_pool_size {
        return Err(ContractError::PoolSizeExceeded {
            available: config.swap_pool_size - state.total_swapped,
        });
    }

    user.swapped_in += swapped_in;
    user.swapped_out += swapped_out;
    state.total_swapped += swapped_out;

    user::store(deps.storage, sender, &user)?;
    state::store(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("swapped_in", swapped_in.to_string())
        .add_attribute("swapped_out", swapped_out.to_string()))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint256,
) -> Result<Response, ContractError> {
    // xyk
    let sender = &deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
    let config = config::read(deps.storage).load().unwrap();
    for strategy in config.distribution_strategy.iter() {
        match strategy {
            Strategy::Lockup { release_time, .. } => {
                if release_time < &env.block.time.seconds() {
                    return Err(ContractError::NotAllowWithdrawAfterRelease {});
                }
            }
            Strategy::Vesting {
                release_start_time, ..
            } => {
                if release_start_time < &env.block.time.seconds() {
                    return Err(ContractError::NotAllowWithdrawAfterRelease {});
                }
            }
        }
    }
    let mut user = user::read(deps.storage, sender).unwrap();
    let mut state = state::read(deps.storage).load().unwrap();

    if !user.swapped_out_claimed.is_zero() {
        return Err(ContractError::NotAllowWithdrawAfterClaim {});
    }

    if user.swapped_in < amount * config.price {
        return Err(ContractError::WithdrawAmountExceeded {
            available: user.swapped_in,
        });
    }

    let withdraw_amount = vpool::calculate_withdraw_amount(&state, &amount)?;
    let penalty = (amount * config.price) - withdraw_amount;

    user.swapped_out = user.swapped_out - amount;
    user.swapped_in = user.swapped_in - (amount * config.price);
    state.total_swapped = state.total_swapped - amount;
    state.liq_x = state.liq_x - withdraw_amount;
    state.liq_y += amount;

    user::store(deps.storage, sender, &user)?;
    state::store(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: state.x_denom.clone(),
                    amount: withdraw_amount.into(),
                },
            )?],
        }))
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.beneficiary,
            amount: vec![deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: state.x_denom,
                    amount: penalty.into(),
                },
            )?],
        }))
        .add_attribute("action", "withdraw")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("amount", withdraw_amount.to_string())
        .add_attribute("penalty", penalty.to_string()))
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let sender = &deps.api.addr_canonicalize(info.sender.as_str()).unwrap();
    let mut user = user::read(deps.storage, sender).unwrap();
    let mut state = state::read(deps.storage).load().unwrap();

    let claimable_token = strategy::claimable_token_of(
        deps.as_ref(),
        env.block.time.seconds(),
        info.sender.to_string(),
    )?;

    user.swapped_out_claimed += claimable_token;
    state.total_claimed += claimable_token;

    user::store(deps.storage, sender, &user)?;
    state::store(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.y_addr,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: claimable_token.into(),
            })?,
            funds: vec![],
        }))
        .add_attribute("action", "claim")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("amount", claimable_token.to_string()))
}

const EARN_LOCK_PERIOD: u64 = 86400 * 7;

pub fn earn(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let state = state::read(deps.storage).load().unwrap();
    let config = config::read(deps.storage).load().unwrap();
    if config.beneficiary != info.sender {
        return Err(ContractError::Unauthorized {
            action: "earn".to_string(),
            expected: config.beneficiary,
            actual: info.sender.to_string(),
        });
    }

    if env.block.time.seconds() < config.finish + EARN_LOCK_PERIOD {
        return Err(ContractError::NotAllowEarnBeforeLockPeriod {});
    }

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.beneficiary,
            amount: vec![deduct_tax(
                deps.as_ref(),
                deps.querier
                    .query_balance(env.contract.address, state.x_denom)
                    .unwrap(),
            )?],
        }))
        .add_attribute("action", "earn")
        .add_attribute("sender", info.sender.to_string()))
}
