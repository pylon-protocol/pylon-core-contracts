use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Api, Extern, HumanAddr, MigrateResponse, MigrateResult, Querier, Storage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::config::KEY_CONFIG;
use crate::state::{config, vpool};
use cosmwasm_storage::Singleton;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NewConfig {
    pub manager: HumanAddr,
    pub refund_denom: String,
    pub base_price: Decimal256,
}

pub fn migration<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>) -> MigrateResult {
    let config = config::read(&deps.storage).unwrap();
    let vpool = vpool::read(&deps.storage).unwrap();

    Singleton::new(&mut deps.storage, KEY_CONFIG)
        .save(&NewConfig {
            manager: config.owner.clone(),
            refund_denom: vpool.x_denom.to_string(),
            base_price: config.base_price.clone(),
        })
        .unwrap();

    Ok(MigrateResponse::default())
}
