use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage};

use crate::state;

fn check_owner<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, env: &Env) -> StdResult<()> {
    let config = state::read_config(&deps.storage)?;
    if deps
        .api
        .human_address(&config.owner)?
        .ne(&env.message.sender)
    {
        return Err(StdError::unauthorized());
    }
    Ok(())
}

pub fn set_deposit_availability<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    availability: bool,
) -> StdResult<HandleResponse> {
    check_owner(deps, &env)?;

    let mut config: state::Config = state::read_config(&deps.storage)?;

    config.open_deposit = availability;

    state::store_config(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
}

pub fn set_withdraw_availability<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    availability: bool,
) -> StdResult<HandleResponse> {
    check_owner(deps, &env)?;

    let mut config: state::Config = state::read_config(&deps.storage)?;

    config.open_withdraw = availability;

    state::store_config(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
}

pub fn set_claim_availability<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    availability: bool,
) -> StdResult<HandleResponse> {
    check_owner(deps, &env)?;

    let mut config: state::Config = state::read_config(&deps.storage)?;

    config.open_claim = availability;

    state::store_config(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
}
