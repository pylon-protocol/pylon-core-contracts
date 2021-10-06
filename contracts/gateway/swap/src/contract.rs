#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use pylon_gateway::swap_msg::{ConfigureMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::error::ContractError;
use crate::handler::configure as ConfigHandler;
use crate::handler::execute as ExecHandler;
use crate::handler::migrate as MigrateHandler;
use crate::handler::query as QueryHandler;
use crate::state::{config, state};

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    config::store(deps.storage).save(&config::Config {
        owner: info.sender.to_string(),
        beneficiary: msg.beneficiary,
        price: msg.price,
        start: msg.start,
        finish: msg.start + msg.period,
        cap_strategy: msg.cap_strategy,
        distribution_strategy: msg.distribution_strategy,
        whitelist_enabled: msg.whitelist_enabled,
        swap_pool_size: msg.swap_pool_size,
    })?;

    state::store(deps.storage).save(&state::State {
        total_swapped: Uint256::zero(),
        total_claimed: Uint256::zero(),
        x_denom: msg.pool_x_denom,
        y_addr: msg.pool_y_addr,
        liq_x: msg.pool_liq_x,
        liq_y: msg.pool_liq_y,
    })?;

    Ok(Response::default())
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
        ExecuteMsg::Configure(cfg_msg) => {
            let config = config::read(deps.storage).load().unwrap();
            if config.owner != info.sender {
                return Err(ContractError::Unauthorized {
                    action: stringify!(cfg_msg).to_string(),
                    expected: config.owner,
                    actual: info.sender.to_string(),
                });
            }

            match cfg_msg {
                ConfigureMsg::Swap {
                    owner,
                    beneficiary,
                    cap_strategy,
                    whitelist_enabled,
                } => ConfigHandler::swap(
                    deps,
                    env,
                    info,
                    owner,
                    beneficiary,
                    cap_strategy,
                    whitelist_enabled,
                ),
                ConfigureMsg::Pool {
                    x_denom,
                    y_addr,
                    liq_x,
                    liq_y,
                } => ConfigHandler::pool(deps, env, info, x_denom, y_addr, liq_x, liq_y),
                ConfigureMsg::Whitelist {
                    whitelist,
                    candidates,
                } => ConfigHandler::whitelist(deps, env, info, whitelist, candidates),
            }
        }
        ExecuteMsg::Deposit {} => ExecHandler::deposit(deps, env, info),
        ExecuteMsg::Withdraw { amount } => ExecHandler::withdraw(deps, env, info, amount),
        ExecuteMsg::Claim {} => ExecHandler::claim(deps, env, info),
        ExecuteMsg::Earn {} => ExecHandler::earn(deps, env, info),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::BalanceOf { owner } => QueryHandler::balance_of(deps, owner),
        QueryMsg::IsWhitelisted { address } => QueryHandler::is_whitelisted(deps, address),
        QueryMsg::AvailableCapOf { address } => QueryHandler::available_cap_of(deps, address),
        QueryMsg::ClaimableTokenOf { address } => {
            QueryHandler::claimable_token_of(deps, env, address)
        }
        QueryMsg::TotalSupply {} => QueryHandler::total_supply(deps),
        QueryMsg::CurrentPrice {} => QueryHandler::current_price(deps),
        QueryMsg::SimulateWithdraw { amount, address } => {
            QueryHandler::simulate_withdraw(deps, address, amount)
        }
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
