use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    Api, CanonicalAddr, Env, Extern, MigrateResponse, MigrateResult, Querier, Storage,
};
use cosmwasm_storage::ReadonlySingleton;
use pylon_gateway::pool_msg::MigrateMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Mul;

use crate::state::time_range::TimeRange;
use crate::state::{config, reward, user};

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: MigrateMsg,
) -> MigrateResult {
    match msg {
        MigrateMsg::V1 {} => migrate_from_v1(deps, env),
        MigrateMsg::V1Temp {} => migrate_from_v1_temp(deps, env),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct V1Config {
    pub owner: CanonicalAddr,
    pub share_token: CanonicalAddr,
    pub reward_token: CanonicalAddr,
    pub start_time: u64,
    pub cliff_time: u64,
    pub finish_time: u64,
    pub reward_rate: Decimal256,
}

fn migrate_from_v1<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _: Env,
) -> MigrateResult {
    let legacy_config: V1Config = ReadonlySingleton::new(&deps.storage, config::KEY_CONFIG)
        .load()
        .unwrap();

    config::store(
        &mut deps.storage,
        &config::Config {
            owner: deps.api.human_address(&legacy_config.owner).unwrap(),
            share_token: deps.api.human_address(&legacy_config.share_token).unwrap(),
            deposit_config: config::DepositConfig {
                time: TimeRange {
                    start: legacy_config.start_time,
                    finish: 0,
                    inverse: false,
                },
                user_cap: Uint256::zero(),
                total_cap: Uint256::zero(),
            },
            withdraw_time: vec![TimeRange {
                start: legacy_config.start_time,
                finish: legacy_config.finish_time,
                inverse: true,
            }],
            reward_token: deps.api.human_address(&legacy_config.reward_token).unwrap(),
            claim_time: TimeRange {
                start: legacy_config.cliff_time,
                finish: 0,
                inverse: false,
            },
            distribution_config: config::DistributionConfig {
                time: TimeRange {
                    start: legacy_config.start_time,
                    finish: legacy_config.finish_time,
                    inverse: false,
                },
                reward_rate: legacy_config.reward_rate,
                total_reward_amount: Uint256::from(
                    legacy_config.finish_time - legacy_config.start_time,
                )
                .mul(legacy_config.reward_rate),
            },
        },
    )
    .unwrap();

    config::read(&deps.storage).unwrap();
    reward::read(&deps.storage).unwrap();
    user::batch_read(deps, None, None).unwrap();

    Ok(MigrateResponse::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct V1TempConfig {
    pub owner: CanonicalAddr,
    pub share_token: CanonicalAddr,
    pub reward_token: CanonicalAddr,
    pub start_time: u64,
    pub cliff_time: u64,
    pub finish_time: u64,
    pub temp_withdraw_start_time: u64,
    pub temp_withdraw_finish_time: u64,
    pub reward_rate: Decimal256,
}

fn migrate_from_v1_temp<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _: Env,
) -> MigrateResult {
    let legacy_config: V1TempConfig = ReadonlySingleton::new(&deps.storage, config::KEY_CONFIG)
        .load()
        .unwrap();

    config::store(
        &mut deps.storage,
        &config::Config {
            owner: deps.api.human_address(&legacy_config.owner).unwrap(),
            share_token: deps.api.human_address(&legacy_config.share_token).unwrap(),
            deposit_config: config::DepositConfig {
                time: TimeRange {
                    start: legacy_config.start_time,
                    finish: 0,
                    inverse: false,
                },
                user_cap: Uint256::zero(),
                total_cap: Uint256::zero(),
            },
            withdraw_time: vec![
                TimeRange {
                    start: legacy_config.start_time,
                    finish: legacy_config.finish_time,
                    inverse: true,
                },
                TimeRange {
                    start: legacy_config.temp_withdraw_start_time,
                    finish: legacy_config.temp_withdraw_finish_time,
                    inverse: false,
                },
            ],
            reward_token: deps.api.human_address(&legacy_config.reward_token).unwrap(),
            claim_time: TimeRange {
                start: legacy_config.cliff_time,
                finish: 0,
                inverse: false,
            },
            distribution_config: config::DistributionConfig {
                time: TimeRange {
                    start: legacy_config.start_time,
                    finish: legacy_config.finish_time,
                    inverse: false,
                },
                reward_rate: legacy_config.reward_rate,
                total_reward_amount: Uint256::from(
                    legacy_config.finish_time - legacy_config.start_time,
                )
                .mul(legacy_config.reward_rate),
            },
        },
    )
    .unwrap();

    config::read(&deps.storage).unwrap();
    reward::read(&deps.storage).unwrap();
    user::batch_read(deps, None, None).unwrap();

    Ok(MigrateResponse::default())
}
