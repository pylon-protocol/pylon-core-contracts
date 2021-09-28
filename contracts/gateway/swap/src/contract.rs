#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use pylon_gateway::swap_msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use std::ops::Add;

use crate::error::ContractError;
use crate::handler::execute as ExecHandler;
use crate::handler::migrate as MigrateHandler;
use crate::handler::query as QueryHandler;
use crate::state::{config, state, vpool};

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    config::store(
        deps.storage,
        &config::Config {
            this: env.contract.address.clone(),
            owner: env.message.sender,
            beneficiary: msg.beneficiary,
            base_price: msg.base_price,
            min_user_cap: msg.min_user_cap,
            max_user_cap: msg.max_user_cap,
            staking_contract: msg.staking_contract,
            min_stake_amount: msg.min_stake_amount,
            max_stake_amount: msg.max_stake_amount,
            additional_cap_per_token: msg.additional_cap_per_token,
            total_sale_amount: msg.total_sale_amount,
            start: msg.start,
            finish: msg.start.add(msg.period),
        },
    )?;

    state::store(
        deps.storage,
        &state::State {
            total_supply: Uint256::zero(),
        },
    )?;

    vpool::store(
        deps.storage,
        &vpool::VirtualPool {
            x_denom: msg.pool_x_denom,
            y_addr: deps.api.canonical_address(&msg.pool_y_addr)?,
            liq_x: msg.pool_liq_x,
            liq_y: msg.pool_liq_y,
        },
    )?;

    Ok(InitResponse::default())
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Configure {
            total_sale_amount,
            min_user_cap,
            max_user_cap,
        } => ExecHandler::configure(deps, env, total_sale_amount, min_user_cap, max_user_cap),
        ExecuteMsg::Deposit {} => ExecHandler::deposit(deps, env),
        ExecuteMsg::Withdraw { amount } => ExecHandler::withdraw(deps, env, amount),
        ExecuteMsg::Earn {} => ExecHandler::earn(deps, env),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::BalanceOf { owner } => QueryHandler::balance_of(deps, owner),
        QueryMsg::AvailableCapOf { address } => QueryHandler::available_cap_of(deps, address),
        QueryMsg::TotalSupply {} => QueryHandler::total_supply(deps),
        QueryMsg::CurrentPrice {} => QueryHandler::current_price(deps),
        QueryMsg::SimulateWithdraw { amount } => QueryHandler::simulate_withdraw(deps, amount),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg::Refund {} => MigrateHandler::refund(deps, env),
        MigrateMsg::General {} => Ok(Response::default()),
    }
}
