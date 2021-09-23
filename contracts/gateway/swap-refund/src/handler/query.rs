use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Coin, Extern, HumanAddr, Querier, StdResult, Storage,
};
use pylon_utils::tax::compute_tax;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Add;

use crate::state::{config, user};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    manager: HumanAddr,
    refund_denom: String,
    base_price: Decimal256,
}

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = config::read(&deps.storage).unwrap();

    Ok(to_binary(&ConfigResponse {
        manager: config.manager,
        refund_denom: config.refund_denom,
        base_price: config.base_price,
    })
    .unwrap())
}

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
    let users = user::batch_read(deps, start_after, limit).unwrap();

    let mut buyers: Vec<Buyer> = Vec::new();
    for (address, user) in users.iter() {
        buyers.push(Buyer {
            address: address.clone(),
            amount: user.amount,
        });
    }

    Ok(to_binary(&BuyersResponse { buyers }).unwrap())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulateResponse {
    pub buyers: Vec<Buyer>,
    pub total_tax: Uint256,
}

pub fn simulate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let config = config::read(&deps.storage).unwrap();
    let users = user::batch_read(deps, start_after, limit).unwrap();

    let mut total_tax = Uint256::zero();
    let mut buyers: Vec<Buyer> = Vec::new();
    for (address, user) in users.iter() {
        buyers.push(Buyer {
            address: address.clone(),
            amount: user.amount,
        });

        total_tax = total_tax.add(
            compute_tax(
                deps,
                &Coin {
                    denom: config.refund_denom.clone(),
                    amount: user.amount.into(),
                },
            )
            .unwrap(),
        );
    }

    Ok(to_binary(&SimulateResponse { buyers, total_tax }).unwrap())
}
