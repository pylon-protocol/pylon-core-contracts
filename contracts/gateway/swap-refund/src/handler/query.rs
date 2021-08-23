use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, StdResult, Storage,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::user;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Buyer {
    pub address: HumanAddr,
    pub amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BuyersResponse {
    pub buyers: Vec<Buyer>,
}

pub fn buyers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let users = user::batch_read(&deps, Option::from(start_after), Option::from(limit)).unwrap();

    let mut buyers: Vec<Buyer> = Vec::new();
    for (address, user) in users.iter() {
        buyers.push(Buyer {
            address: address.clone(),
            amount: user.amount.clone(),
        });
    }

    Ok(to_binary(&BuyersResponse { buyers }).unwrap())
}
