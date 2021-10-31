use cosmwasm_bignumber::Uint256;
use cosmwasm_std::*;
use cw20::Cw20ExecuteMsg;
use pylon_gateway::cap_strategy_msg::QueryMsg;
use pylon_gateway::cap_strategy_resp;
use std::ops::{Add, Sub};

use crate::error::ContractError;
use crate::handler::util_staking;
use crate::state::{config, reward, user};

pub fn update(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target: Option<String>,
) -> Result<Response, ContractError> {
    let config = config::read(deps.storage).unwrap().distribution_config;
    let applicable_reward_time = config.applicable_reward_time(env.block.time.seconds());

    // reward
    let mut reward = reward::read(deps.storage).unwrap();
    reward.reward_per_token_stored =
        reward
            .reward_per_token_stored
            .add(util_staking::calculate_reward_per_token(
                deps.as_ref(),
                &reward,
                applicable_reward_time,
            )?);
    reward.last_update_time = applicable_reward_time;
    reward::store(deps.storage, &reward).unwrap();

    // user
    let mut resp = Response::new()
        .add_attribute("action", "update")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("stored_rpt", reward.reward_per_token_stored.to_string());

    if let Some(target) = target {
        let t = deps.api.addr_canonicalize(target.as_str()).unwrap();
        let mut user = user::read(deps.storage, &t).unwrap();

        user.reward =
            util_staking::calculate_rewards(deps.as_ref(), &reward, &user, applicable_reward_time)?;
        user.reward_per_token_paid = reward.reward_per_token_stored;

        user::store(deps.storage, &t, &user).unwrap();
        resp = resp
            .add_attribute("target", target)
            .add_attribute("reward", user.reward.to_string())
    }

    Ok(resp)
}

pub fn deposit_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint256,
) -> Result<Response, ContractError> {
    if env.contract.address.ne(&info.sender) {
        return Err(ContractError::Unauthorized {
            action: "deposit_internal".to_string(),
            expected: env.contract.address.to_string(),
            actual: info.sender.to_string(),
        });
    }

    let config = config::read(deps.storage).unwrap();
    // check deposit config & temporary deposit config
    config.check_deposit_time(&env)?;

    let mut reward = reward::read(deps.storage).unwrap();
    let mut user = user::read(
        deps.storage,
        &deps.api.addr_canonicalize(sender.as_str()).unwrap(),
    )
    .unwrap();

    // check total cap
    reward.total_deposit = reward.total_deposit.add(amount);
    if config.deposit_config.total_cap.ne(&Uint256::zero())
        && reward.total_deposit.gt(&config.deposit_config.total_cap)
    {
        return Err(ContractError::DepositTotalCapExceeded {
            cap: config.deposit_config.total_cap,
        });
    }

    // check user cap
    user.amount = user.amount.add(amount);
    if config.deposit_config.user_cap.ne(&Uint256::zero())
        && user.amount.gt(&config.deposit_config.user_cap)
    {
        return Err(ContractError::DepositUserCapExceeded {
            cap: config.deposit_config.user_cap,
        });
    }

    if let Some(strategy) = config.cap_strategy {
        let resp: cap_strategy_resp::AvailableCapOfResponse = deps.querier.query_wasm_smart(
            strategy,
            &QueryMsg::AvailableCapOf {
                address: sender.clone(),
                amount: user.amount,
            },
        )?;

        if let Some(v) = resp.amount {
            if v < user.amount {
                return Err(ContractError::DepositUserCapExceeded { cap: v });
            }
        }
    }

    reward::store(deps.storage, &reward).unwrap();
    user::store(
        deps.storage,
        &deps.api.addr_canonicalize(sender.as_str()).unwrap(),
        &user,
    )
    .unwrap();

    Ok(Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("deposit_amount", amount.to_string()))
}

pub fn withdraw_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint256,
) -> Result<Response, ContractError> {
    if env.contract.address.ne(&info.sender) {
        return Err(ContractError::Unauthorized {
            action: "withdraw_internal".to_string(),
            expected: env.contract.address.to_string(),
            actual: info.sender.to_string(),
        });
    }

    let config = config::read(deps.storage).unwrap();
    // check withdraw config & temporary withdraw config
    config.check_withdraw_time(&env)?;

    let owner = deps.api.addr_canonicalize(sender.as_str()).unwrap();
    let mut reward = reward::read(deps.storage).unwrap();
    let mut user = user::read(deps.storage, &owner).unwrap();

    if amount > user.amount {
        return Err(ContractError::WithdrawAmountExceeded { amount });
    }

    reward.total_deposit = reward.total_deposit.sub(amount);
    user.amount = user.amount.sub(amount);

    reward::store(deps.storage, &reward).unwrap();
    user::store(deps.storage, &owner, &user).unwrap();

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.share_token,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender,
                amount: amount.into(),
            })
            .unwrap(),
            funds: vec![],
        }))
        .add_attribute("action", "withdraw")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("withdraw_amount", amount.to_string()))
}

pub fn claim_internal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
) -> Result<Response, ContractError> {
    if env.contract.address.ne(&info.sender) {
        return Err(ContractError::Unauthorized {
            action: "claim_internal".to_string(),
            expected: env.contract.address.to_string(),
            actual: info.sender.to_string(),
        });
    }

    let config = config::read(deps.storage).unwrap();
    // check claim config
    config.check_claim_time(&env)?;

    let owner = deps.api.addr_canonicalize(sender.as_str()).unwrap();
    let mut user = user::read(deps.storage, &owner).unwrap();

    let claim_amount = user.reward;
    user.reward = Uint256::zero();
    user::store(deps.storage, &owner, &user).unwrap();

    let config = config::read(deps.storage).unwrap();

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.reward_token,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: sender,
                amount: claim_amount.into(),
            })
            .unwrap(),
            funds: vec![],
        }))
        .add_attribute("action", "claim")
        .add_attribute("sender", info.sender.to_string())
        .add_attribute("claim_amount", claim_amount.to_string()))
}
