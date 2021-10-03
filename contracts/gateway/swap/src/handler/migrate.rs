use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{DepsMut, Env, Response};
use cosmwasm_storage::Singleton;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::state::config::KEY_CONFIG;
use crate::state::{config, state};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NewRefundConfig {
    pub manager: String,
    pub swap_price: Decimal256,
    pub refund_denom: String,
}

pub fn refund(deps: DepsMut, _: Env) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).load().unwrap();
    let state = state::read(deps.storage).load().unwrap();

    Singleton::new(deps.storage, KEY_CONFIG)
        .save(&NewRefundConfig {
            manager: config.owner.clone(),
            swap_price: config.price,
            refund_denom: state.x_denom,
        })
        .unwrap();

    Ok(Response::default())
}
