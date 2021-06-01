use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage};

use crate::state;

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // 1:1
    Ok(HandleResponse::default())
}

pub fn withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // xyk
    Ok(HandleResponse::default())
}

fn check_owner<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<()> {
    let config: state::Config = state::read_config(&deps.storage)?;
    if config.owner.ne(&env.message.sender) {
        return Err(StdError::unauthorized());
    }
    Ok(())
}

pub fn earn<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    check_owner(deps, env)?;

    Ok(HandleResponse::default())
}
