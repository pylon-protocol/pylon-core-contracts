use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult,
    Querier, StdResult, Storage,
};
use pylon_gateway::swap_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};
use std::ops::Add;

use crate::handler::execute as ExecHandler;
use crate::handler::query as QueryHandler;
use crate::migration::migration;
use crate::state::{config, state, user, vpool};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    config::store(
        &mut deps.storage,
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
        &mut deps.storage,
        &state::State {
            total_supply: Uint256::zero(),
        },
    )?;

    vpool::store(
        &mut deps.storage,
        &vpool::VirtualPool {
            x_denom: msg.pool_x_denom,
            y_addr: deps.api.canonical_address(&msg.pool_y_addr)?,
            liq_x: msg.pool_liq_x,
            liq_y: msg.pool_liq_y,
        },
    )?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Configure {
            total_sale_amount,
            min_user_cap,
            max_user_cap,
        } => ExecHandler::configure(deps, env, total_sale_amount, min_user_cap, max_user_cap),
        HandleMsg::Deposit {} => ExecHandler::deposit(deps, env),
        HandleMsg::Withdraw { amount } => ExecHandler::withdraw(deps, env, amount),
        HandleMsg::Earn {} => ExecHandler::earn(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::BalanceOf { owner } => QueryHandler::balance_of(deps, owner),
        QueryMsg::AvailableCapOf { address } => QueryHandler::available_cap_of(deps, address),
        QueryMsg::TotalSupply {} => QueryHandler::total_supply(deps),
        QueryMsg::CurrentPrice {} => QueryHandler::current_price(deps),
        QueryMsg::SimulateWithdraw { amount } => QueryHandler::simulate_withdraw(deps, amount),
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _: Env,
    _: MigrateMsg,
) -> MigrateResult {
    migration(deps)
}
