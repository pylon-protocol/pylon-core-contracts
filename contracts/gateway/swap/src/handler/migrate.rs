use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Api, DepsMut, Env, Querier, Response, Storage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::state::config::KEY_CONFIG;
use crate::state::{config, vpool};
use cosmwasm_storage::Singleton;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NewRefundConfig {
    pub manager: String,
    pub refund_denom: String,
    pub base_price: Decimal256,
}

pub fn refund(deps: DepsMut, _: Env) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).unwrap();
    let vpool = vpool::read(deps.storage).unwrap();

    Singleton::new(deps.storage, KEY_CONFIG)
        .save(&NewRefundConfig {
            manager: config.owner.clone(),
            refund_denom: vpool.x_denom,
            base_price: config.base_price,
        })
        .unwrap();

    Ok(Response::default())
}
