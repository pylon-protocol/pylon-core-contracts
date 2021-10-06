use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::error::ContractError;
use crate::state::{config, state, user};
use cosmwasm_bignumber::Uint256;

pub fn swap(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    owner: Option<String>,
    beneficiary: Option<String>,
    cap_strategy: Option<String>,
    whitelist_enabled: Option<bool>,
) -> Result<Response, ContractError> {
    let mut config = config::read(deps.storage).load().unwrap();

    if let Some(v) = owner {
        config.owner = v;
    }

    if let Some(v) = beneficiary {
        config.beneficiary = v;
    }

    config.cap_strategy = cap_strategy;

    if let Some(v) = whitelist_enabled {
        config.whitelist_enabled = v;
    }

    config::store(deps.storage).save(&config).unwrap();

    Ok(Response::default())
}

pub fn pool(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    x_denom: Option<String>,
    y_addr: Option<String>,
    liq_x: Option<Uint256>,
    liq_y: Option<Uint256>,
) -> Result<Response, ContractError> {
    let mut state = state::read(deps.storage).load().unwrap();

    if let Some(v) = x_denom {
        state.x_denom = v;
    }
    if let Some(v) = y_addr {
        state.y_addr = v;
    }
    if let Some(v) = liq_x {
        state.liq_x = v;
    }
    if let Some(v) = liq_y {
        state.liq_y = v;
    }

    state::store(deps.storage).save(&state).unwrap();

    Ok(Response::default())
}

pub fn whitelist(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    whitelist: bool,
    candidates: Vec<String>,
) -> Result<Response, ContractError> {
    for candidate in candidates.iter() {
        let address = &deps.api.addr_canonicalize(candidate.as_str()).unwrap();
        let mut user = user::read(deps.storage, address).unwrap();

        if user.whitelisted {
            continue;
        } else {
            user.whitelisted = whitelist;
            user::store(deps.storage, address, &user).unwrap();
        }
    }

    Ok(Response::default())
}
