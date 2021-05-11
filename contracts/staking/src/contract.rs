use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128,
};

use crate::handler_exec as ExecHandler;
use crate::handler_query as QueryHandler;
use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    state::store_config(
        &mut deps.storage,
        &state::Config {
            owner: deps.api.canonical_address(&env.message.sender)?,
            dp_token: deps.api.canonical_address(&msg.dp_token)?,
            reward_token: deps.api.canonical_address(&msg.reward_token)?,
            start_time: 0,
            finish_time: 0,
            open_deposit: false,
            open_withdraw: false,
            open_claim: false,
            reward_rate: Decimal256::zero(),
        },
    )?;

    state::store_reward(
        &mut deps.storage,
        &state::Reward {
            total_deposit: Uint128::zero(),
            last_update_time: 0,
            reward_per_token_stored: Decimal256::zero(),
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
        HandleMsg::Receive(msg) => ExecHandler::receive(deps, &env, msg),
        HandleMsg::Update {} => ExecHandler::update(deps, &env, None),
        HandleMsg::Withdraw { amount } => ExecHandler::withdraw(deps, &env, amount),
        HandleMsg::Claim {} => ExecHandler::claim(deps, &env),
        HandleMsg::Exit {} => ExecHandler::exit(deps, &env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::Reward {} => QueryHandler::reward(deps),
        QueryMsg::BalanceOf { owner } => QueryHandler::balance_of(deps, owner),
        QueryMsg::ClaimableReward { owner, timestamp } => {
            QueryHandler::claimable_reward(deps, owner, timestamp)
        }
    }
}
