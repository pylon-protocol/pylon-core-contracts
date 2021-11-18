use cosmwasm_std::{to_binary, Deps};
use pylon_token::gov_resp::ConfigResponse;

use crate::queries::QueryResult;
use crate::state::config::Config;

pub fn query_config(deps: Deps) -> QueryResult {
    let config = Config::load(deps.storage)?;
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
