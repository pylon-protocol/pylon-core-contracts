use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, StdError, StdResult, Storage,
};
use pylon_core::exchange_rate_resp as resp;
use std::ops::{Div, Mul, Sub};

use crate::state;

pub fn exchange_rate_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token: &HumanAddr,
    blocktime: Option<u64>,
) -> StdResult<Binary> {
    let token_addr: CanonicalAddr = deps.api.canonical_address(token)?;

    let token: state::Token = state::read_token(&deps.storage, &token_addr)?;
    if token.status.ne(&state::Status::Running) {
        return Err(StdError::unauthorized());
    }

    let mut exchange_rate = token.exchange_rate;

    if let Some(t) = blocktime {
        let elapsed = t.sub(token.last_updated_at);
        if elapsed.gt(&token.epoch_period) || elapsed.eq(&token.epoch_period) {
            let pow_count = elapsed.div(token.epoch_period);
            for _ in 0..pow_count {
                exchange_rate = exchange_rate.mul(token.weight);
            }
        }
    }

    to_binary(&resp::ExchangeRateResponse {
        exchange_rate: token.exchange_rate,
    })
}
