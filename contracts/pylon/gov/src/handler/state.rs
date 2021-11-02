use cosmwasm_std::{to_binary, Binary, Deps};
use pylon_token::gov_resp::StateResponse;

use crate::error::ContractError;
use crate::state::state::state_r;

pub fn query_state(deps: Deps) -> Result<Binary, ContractError> {
    let state = state_r(deps.storage).load()?;
    Ok(to_binary(&StateResponse {
        poll_count: state.poll_count,
        total_share: state.total_share,
        total_deposit: state.total_deposit,
    })?)
}
