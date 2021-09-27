use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Api, CosmosMsg, Env, Extern, HumanAddr, MigrateResponse, MigrateResult, Querier,
    Storage, WasmMsg,
};
use cosmwasm_storage::ReadonlySingleton;
use pylon_gateway::pool_msg::{MigrateMsg, Transfer};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::config::DepositConfig;
use crate::state::time_range::TimeRange;
use crate::state::{config, reward, user};
use cw20::Cw20HandleMsg;

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: MigrateMsg,
) -> MigrateResult {
    match msg {
        MigrateMsg::Legacy { transfer } => migrate_from_legacy(deps, env, transfer),
        MigrateMsg::Common {} => Ok(MigrateResponse::default()),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyDistributionConfig {
    pub time: TimeRange,
    pub reward_rate: Decimal256,
    pub total_reward_amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyConfig {
    pub owner: HumanAddr,
    // share
    pub share_token: HumanAddr,
    pub deposit_config: DepositConfig,
    pub withdraw_time: Vec<TimeRange>,
    // reward
    pub reward_token: HumanAddr,
    pub claim_time: TimeRange,
    pub distribution_config: LegacyDistributionConfig,
}

fn migrate_from_legacy<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _: Env,
    transfer: Option<Transfer>,
) -> MigrateResult {
    let legacy_config: LegacyConfig = ReadonlySingleton::new(&deps.storage, config::KEY_CONFIG)
        .load()
        .unwrap();

    config::store(
        &mut deps.storage,
        &config::Config {
            owner: legacy_config.owner,
            share_token: legacy_config.share_token,
            deposit_config: legacy_config.deposit_config,
            withdraw_time: legacy_config.withdraw_time,
            reward_token: legacy_config.reward_token,
            claim_time: legacy_config.claim_time,
            distribution_config: config::DistributionConfig {
                time: legacy_config.distribution_config.time,
                reward_rate: legacy_config.distribution_config.reward_rate,
            },
        },
    )
    .unwrap();

    config::read(&deps.storage).unwrap();
    reward::read(&deps.storage).unwrap();
    user::batch_read(deps, None, None).unwrap();

    if let Some(transfer) = transfer {
        let config = config::read(&deps.storage).unwrap();
        return Ok(MigrateResponse {
            messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.reward_token,
                msg: to_binary(&Cw20HandleMsg::Transfer {
                    recipient: transfer.to,
                    amount: transfer.amount,
                })
                .unwrap(),
                send: vec![],
            })],
            log: vec![],
            data: None,
        });
    }

    Ok(MigrateResponse::default())
}
