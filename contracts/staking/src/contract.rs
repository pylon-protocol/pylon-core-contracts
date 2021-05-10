use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage,
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
        HandleMsg::Receive(msg) => ExecHandler::receive(deps, env, msg),
        HandleMsg::Update {} => ExecHandler::update(deps, env),
        HandleMsg::Withdraw {} => ExecHandler::withdraw(deps, env),
        HandleMsg::Exit {} => ExecHandler::exit(deps, env),
        HandleMsg::Claim {} => ExecHandler::claim(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {}
}
