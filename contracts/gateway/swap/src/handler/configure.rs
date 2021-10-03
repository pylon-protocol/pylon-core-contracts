use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::error::ContractError;
use crate::state::{config, user};

pub fn whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    whitelist: bool,
    candidates: Vec<String>,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).load().unwrap();
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {
            action: "configure_whitelist".to_string(),
            expected: config.owner,
            actual: info.sender.to_string(),
        });
    }

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
