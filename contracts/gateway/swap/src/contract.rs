use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, MigrateResponse, MigrateResult,
    Querier, StdResult, Storage,
};

use crate::handler::execute as ExecHandler;
use crate::handler::query as QueryHandler;
use crate::state;
use cosmwasm_bignumber::Uint256;
use pylon_gateway::swap_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};
use std::ops::Add;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    state::store_config(
        &mut deps.storage,
        &state::Config {
            this: env.contract.address.clone(),
            owner: env.message.sender,
            beneficiary: msg.beneficiary,
            start: msg.start,
            finish: msg.start.add(msg.period),
            price: msg.price,
            max_cap: msg.max_cap,
            total_sale_amount: msg.total_sale_amount,
        },
    )?;

    state::store_reward(
        &mut deps.storage,
        &state::Reward {
            total_supply: Uint256::zero(),
        },
    )?;

    state::store_vpool(
        &mut deps.storage,
        &state::VirtualPool {
            x_denom: msg.x_denom,
            y_addr: deps.api.canonical_address(&msg.y_addr)?,
            liq_x: msg.liq_x,
            liq_y: msg.liq_y,
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
        QueryMsg::TotalSupply {} => QueryHandler::total_supply(deps),
        QueryMsg::CurrentPrice {} => QueryHandler::current_price(deps),
        QueryMsg::SimulateWithdraw { amount } => QueryHandler::simulate_withdraw(deps, amount),
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _: &mut Extern<S, A, Q>,
    _: Env,
    _: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
