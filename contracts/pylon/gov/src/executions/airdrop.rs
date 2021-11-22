use crate::constant::MAX_QUERY_LIMIT;
use cosmwasm_std::{
    to_binary, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use pylon_token::gov_msg::{AirdropMsg, ExecuteMsg};
use std::cmp::max;

use crate::error::ContractError;
use crate::executions::ExecuteResult;
use crate::state::airdrop::{
    Airdrop, Config as AirdropConfig, Reward as AirdropReward, State as AirdropState,
};
use crate::state::bank::TokenManager;
use crate::state::config::Config;
use crate::state::state::State;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    start: u64,
    period: u64,
    reward_token: String,
    reward_amount: Uint128,
) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "airdrop_instantiate");

    let config = Config::load(deps.storage)?;
    if config.owner != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    let mut state = State::load(deps.storage)?;
    let airdrop_id = state.total_airdrop_count;

    Airdrop::save(
        deps.storage,
        &airdrop_id,
        &Airdrop {
            config: AirdropConfig {
                start,
                period,
                reward_token: deps.api.addr_validate(reward_token.as_str())?,
                reward_rate: Decimal::from_ratio(reward_amount, period),
            },
            state: AirdropState {
                last_update_time: start,
                reward_per_token_stored: Decimal::zero(),
            },
        },
    )?;

    state.total_airdrop_count += 1;
    state.airdrop_update_candidates.push(airdrop_id);

    State::save(deps.storage, &state)?;

    Ok(response.add_attributes(vec![
        ("airdrop_id", &airdrop_id.to_string()),
        ("reward_token", &reward_token),
        ("reward_amount", &reward_amount.to_string()),
    ]))
}

pub fn allocate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    airdrop_id: u64,
    recipient: String,
    allocate_amount: Uint128,
) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "airdrop_allocate");

    let config = Config::load(deps.storage)?;
    if config.owner != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    let mut airdrop_reward = AirdropReward::load(
        deps.storage,
        &deps.api.addr_validate(recipient.as_str())?,
        &airdrop_id,
    )?;

    airdrop_reward.reward += allocate_amount;

    AirdropReward::save(
        deps.storage,
        &deps.api.addr_validate(recipient.as_str())?,
        &airdrop_id,
        &airdrop_reward,
    )?;

    Ok(response.add_attributes(vec![
        ("airdrop_id", &airdrop_id.to_string()),
        ("recipient", &recipient),
        ("amount", &allocate_amount.to_string()),
    ]))
}

pub fn deallocate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    airdrop_id: u64,
    recipient: String,
    deallocate_amount: Uint128,
) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "airdrop_deallocate");

    let config = Config::load(deps.storage)?;
    if config.owner != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    let mut airdrop_reward = AirdropReward::load(
        deps.storage,
        &deps.api.addr_validate(recipient.as_str())?,
        &airdrop_id,
    )?;

    if airdrop_reward.reward < deallocate_amount {
        return Err(ContractError::InsufficientReward {});
    }
    airdrop_reward.reward -= deallocate_amount;

    AirdropReward::save(
        deps.storage,
        &deps.api.addr_validate(recipient.as_str())?,
        &airdrop_id,
        &airdrop_reward,
    )?;

    Ok(response.add_attributes(vec![
        ("airdrop_id", &airdrop_id.to_string()),
        ("recipient", &recipient),
        ("amount", &deallocate_amount.to_string()),
    ]))
}

pub fn update(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    target: Option<String>,
) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "airdrop_update");

    let target = target.map(|x| deps.api.addr_validate(x.as_str()).unwrap());
    let state = State::load(deps.storage)?;

    for airdrop_id in state.airdrop_update_candidates.iter() {
        let mut airdrop = match Airdrop::load(deps.storage, airdrop_id) {
            Some(airdrop) => airdrop,
            None => return Err(ContractError::AirdropNotFound {}),
        };
        let applicable_time = airdrop.applicable_time(&env.block);

        airdrop.state.reward_per_token_stored =
            if airdrop.finish() == airdrop.state.last_update_time {
                airdrop.state.reward_per_token_stored // because it's already latest
            } else {
                airdrop.state.reward_per_token_stored
                    + calculate_reward_per_token(
                        &applicable_time,
                        &state.total_share,
                        &airdrop.config.reward_rate,
                        &airdrop.state.last_update_time,
                    )?
            };
        airdrop.state.last_update_time = applicable_time;

        Airdrop::save(deps.storage, airdrop_id, &airdrop)?;

        if let Some(target) = &target {
            let mut airdrop_reward = AirdropReward::load(deps.storage, target, airdrop_id)?;
            let token_manager =
                TokenManager::load(deps.storage, &deps.api.addr_canonicalize(target.as_str())?)?;

            airdrop_reward.reward = calculate_rewards(
                &applicable_time,
                &state.total_share,
                &token_manager.share,
                &airdrop,
                &airdrop_reward,
            )?;
            airdrop_reward.reward_per_token_paid = airdrop.state.reward_per_token_stored;

            AirdropReward::save(deps.storage, target, airdrop_id, &airdrop_reward)?;
        }
    }

    Ok(response.add_attributes(vec![(
        "updated",
        format!("{:?}", state.airdrop_update_candidates),
    )]))
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo, sender: Option<String>) -> ExecuteResult {
    let sender = sender
        .map(|x| deps.api.addr_validate(x.as_str()).unwrap())
        .unwrap_or(info.sender);

    let state = State::load(deps.storage).unwrap();
    let token_manager =
        TokenManager::load(deps.storage, &deps.api.addr_canonicalize(sender.as_str())?)?;

    let airdrop_rewards =
        AirdropReward::load_range(deps.storage, &sender, None, Some(MAX_QUERY_LIMIT), None)?;

    let response = Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::Airdrop(AirdropMsg::Update {
            target: Some(sender.to_string()),
        }))?,
        funds: vec![],
    }));

    Ok(response.add_messages(
        airdrop_rewards
            .iter()
            .map(|(airdrop_id, airdrop_reward)| {
                let mut airdrop = Airdrop::load(deps.storage, airdrop_id).unwrap();
                let applicable_time = airdrop.applicable_time(&env.block);

                airdrop.state.reward_per_token_stored =
                    if airdrop.finish() == airdrop.state.last_update_time {
                        airdrop.state.reward_per_token_stored // because it's already latest
                    } else {
                        airdrop.state.reward_per_token_stored
                            + calculate_reward_per_token(
                                &applicable_time,
                                &state.total_share,
                                &airdrop.config.reward_rate,
                                &airdrop.state.last_update_time,
                            )
                            .unwrap()
                    };
                airdrop.state.last_update_time = applicable_time;

                let mut airdrop_reward = airdrop_reward.clone();
                airdrop_reward.reward = calculate_rewards(
                    &applicable_time,
                    &state.total_share,
                    &token_manager.share,
                    &airdrop,
                    &airdrop_reward,
                )
                .unwrap();
                airdrop_reward.reward_per_token_paid = airdrop.state.reward_per_token_stored;

                (airdrop_id, airdrop_reward)
            })
            .filter(|(_, airdrop_reward)| !airdrop_reward.reward.is_zero())
            .map(|(airdrop_id, _)| {
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::Airdrop(AirdropMsg::ClaimInternal {
                        sender: sender.to_string(),
                        airdrop_id: *airdrop_id,
                    }))
                    .unwrap(),
                    funds: vec![],
                })
            }),
    ))
}

pub fn claim_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    airdrop_id: u64,
) -> ExecuteResult {
    let response = Response::new()
        .add_attribute("action", "airdrop_claim")
        .add_attribute("target", sender.as_str());

    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let sender = deps.api.addr_validate(sender.as_str())?;
    let airdrop_reward = AirdropReward::load(deps.storage, &sender, &airdrop_id)?;
    let airdrop = Airdrop::load(deps.storage, &airdrop_id).unwrap();
    let claim_amount = airdrop_reward.reward;

    AirdropReward::save(
        deps.storage,
        &sender,
        &airdrop_id,
        &AirdropReward {
            reward: Uint128::zero(),
            reward_per_token_paid: airdrop_reward.reward_per_token_paid,
        },
    )?;

    Ok(response
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: airdrop.config.reward_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender.to_string(),
                amount: claim_amount,
            })?,
            funds: vec![],
        }))
        .add_attributes(vec![
            ("token", airdrop.config.reward_token.as_str()),
            ("amount", &claim_amount.to_string()),
        ]))
}

pub fn calculate_reward_per_token(
    timestamp: &u64,
    total_share: &Uint128,
    reward_rate: &Decimal,
    last_update_time: &u64,
) -> StdResult<Decimal> {
    if total_share.is_zero() {
        Ok(Decimal::zero())
    } else {
        Ok(Decimal::from_ratio(
            Uint128::from(max(timestamp, last_update_time) - last_update_time) * *reward_rate,
            *total_share,
        ))
    }
}

pub fn calculate_rewards(
    timestamp: &u64,
    total_share: &Uint128,
    user_share: &Uint128,
    airdrop: &Airdrop,
    airdrop_reward: &AirdropReward,
) -> StdResult<Uint128> {
    let mut rpt = airdrop.state.reward_per_token_stored - airdrop_reward.reward_per_token_paid;

    if airdrop.state.last_update_time != *timestamp {
        rpt = rpt
            + calculate_reward_per_token(
                timestamp,
                total_share,
                &airdrop.config.reward_rate,
                &airdrop.state.last_update_time,
            )?;
    }

    Ok(airdrop_reward.reward + (rpt * *user_share))
}
