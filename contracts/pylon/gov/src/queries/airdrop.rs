use cosmwasm_std::{to_binary, Deps};
use pylon_token::common::OrderBy;
use pylon_token::gov_resp::{AirdropResponse, AirdropsResponse};

use crate::queries::QueryResult;
use crate::state::airdrop::Airdrop;

pub fn query_airdrop(deps: Deps, airdrop_id: u64) -> QueryResult {
    let airdrop = Airdrop::load(deps.storage, &airdrop_id).unwrap();

    Ok(to_binary(&AirdropResponse {
        start: airdrop.config.start,
        period: airdrop.config.period,
        reward_token: airdrop.config.reward_token.to_string(),
        reward_rate: airdrop.config.reward_rate,
    })?)
}

pub fn query_airdrops(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> QueryResult {
    let airdrops = Airdrop::load_range(deps.storage, start_after, limit, order_by)?;

    let airdrop_responses: Vec<(u64, AirdropResponse)> = airdrops
        .iter()
        .map(|(airdrop_id, airdrop)| {
            (
                *airdrop_id,
                AirdropResponse {
                    start: airdrop.config.start,
                    period: airdrop.config.period,
                    reward_token: airdrop.config.reward_token.to_string(),
                    reward_rate: airdrop.config.reward_rate,
                },
            )
        })
        .collect();

    Ok(to_binary(&AirdropsResponse {
        airdrops: airdrop_responses,
    })?)
}
