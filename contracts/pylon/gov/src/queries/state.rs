use cosmwasm_std::{to_binary, Deps};
use pylon_token::gov_resp::StateResponse;

use crate::queries::QueryResult;
use crate::state::state::State;

pub fn query_state(deps: Deps) -> QueryResult {
    let state = State::load(deps.storage)?;

    Ok(to_binary(&StateResponse {
        poll_count: state.poll_count,
        total_share: state.total_share,
        total_deposit: state.total_deposit,
        total_airdrop_count: state.total_airdrop_count,
        airdrop_update_candidates: state.airdrop_update_candidates,
    })?)
}
