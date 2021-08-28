use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Api, CanonicalAddr, Extern, MigrateResponse, MigrateResult, Querier, Storage};
use cosmwasm_storage::ReadonlySingleton;
use pylon_launchpad::lockup_msg::MigrateMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyConfig {
    pub owner: CanonicalAddr,
    pub share_token: CanonicalAddr,
    pub reward_token: CanonicalAddr,
    pub start_time: u64,
    pub cliff_time: u64,
    pub finish_time: u64,
    pub reward_rate: Decimal256,
}

pub fn migration<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    msg: MigrateMsg,
) -> MigrateResult {
    let config: LegacyConfig = ReadonlySingleton::new(&deps.storage, state::KEY_CONFIG)
        .load()
        .unwrap();

    state::store_config(
        &mut deps.storage,
        &state::Config {
            owner: config.owner,
            share_token: config.share_token,
            reward_token: config.reward_token,
            start_time: config.start_time,
            cliff_time: config.cliff_time,
            finish_time: config.finish_time,
            temp_withdraw_start_time: msg.temp_withdraw_start_time,
            temp_withdraw_finish_time: msg.temp_withdraw_finish_time,
            reward_rate: config.reward_rate,
        },
    )
    .unwrap();

    Ok(MigrateResponse::default())
}
