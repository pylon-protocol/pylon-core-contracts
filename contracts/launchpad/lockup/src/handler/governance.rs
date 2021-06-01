use cosmwasm_std::{Api, Extern, HandleResponse, Querier, StdResult, Storage};

use crate::state;

pub fn set_deposit_availability<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    availability: bool,
) -> StdResult<HandleResponse> {
    let mut config: state::Config = state::read_config(&deps.storage)?;

    config.open_deposit = availability;

    state::store_config(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
}

pub fn set_withdraw_availability<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    availability: bool,
) -> StdResult<HandleResponse> {
    let mut config: state::Config = state::read_config(&deps.storage)?;

    config.open_withdraw = availability;

    state::store_config(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
}

pub fn set_claim_availability<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    availability: bool,
) -> StdResult<HandleResponse> {
    let mut config: state::Config = state::read_config(&deps.storage)?;

    config.open_claim = availability;

    state::store_config(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
}
