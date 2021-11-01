use cosmwasm_std::{to_binary, Binary, Decimal, Deps, DepsMut, MessageInfo, Response, Uint128};
use pylon_token::gov::ConfigResponse;

use crate::error::ContractError;
use crate::state::config::{config_r, config_w};

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    timelock_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> Result<Response, ContractError> {
    let api = deps.api;
    config_w(deps.storage).update(|mut config| {
        if config.owner != api.addr_canonicalize(info.sender.as_str())? {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = owner {
            config.owner = api.addr_canonicalize(&owner)?;
        }

        if let Some(quorum) = quorum {
            config.quorum = quorum;
        }

        if let Some(threshold) = threshold {
            config.threshold = threshold;
        }

        if let Some(voting_period) = voting_period {
            config.voting_period = voting_period;
        }

        if let Some(timelock_period) = timelock_period {
            config.timelock_period = timelock_period;
        }

        if let Some(proposal_deposit) = proposal_deposit {
            config.proposal_deposit = proposal_deposit;
        }

        if let Some(period) = snapshot_period {
            config.snapshot_period = period;
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

pub fn query_config(deps: Deps) -> Result<Binary, ContractError> {
    let config = config_r(deps.storage).load()?;
    Ok(to_binary(&ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        pylon_token: deps.api.addr_humanize(&config.pylon_token)?.to_string(),
        quorum: config.quorum,
        threshold: config.threshold,
        voting_period: config.voting_period,
        timelock_period: config.timelock_period,
        proposal_deposit: config.proposal_deposit,
        snapshot_period: config.snapshot_period,
    })?)
}
