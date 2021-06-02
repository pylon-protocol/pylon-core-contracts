use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, StdError, StdResult, Storage,
};

use crate::resp;
use crate::state;
use std::ops::{Div, Mul, Sub};

pub fn exchange_rate_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token: &HumanAddr,
    blocktime: Option<u64>,
) -> StdResult<Binary> {
    let token_addr: CanonicalAddr = deps.api.canonical_address(token)?;

    let token: state::Token = state::read_token(&deps.storage, &token_addr)?;
    if token.status.ne(&state::Status::RUNNING) {
        return Err(StdError::unauthorized());
    }

    let mut exchange_rate = token.exchange_rate.clone();

    if let Some(t) = blocktime {
        let elapsed = t.sub(token.last_updated_at);
        if elapsed.gt(&token.epoch_period) || elapsed.eq(&token.epoch_period) {
            let pow_count = elapsed.div(token.epoch_period);
            for _ in 0..pow_count {
                exchange_rate = exchange_rate.mul(token.weight);
            }
        }
    }

    Ok(to_binary(&resp::ExchangeRateResponse {
        exchange_rate: token.exchange_rate.clone(),
    })?)
}
