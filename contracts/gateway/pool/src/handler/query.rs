use cosmwasm_std::*;
use pylon_gateway::cap_strategy_msg::QueryMsg;
use pylon_gateway::pool_resp as resp;

use crate::handler::util_staking;
use crate::state::{config, reward, user};

pub fn config(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    to_binary(&config) // TODO: marshal config
}

pub fn reward(deps: Deps, _env: Env) -> StdResult<Binary> {
    let reward = reward::read(deps.storage).unwrap();

    to_binary(&resp::RewardResponse {
        total_deposit: reward.total_deposit,
        last_update_time: reward.last_update_time,
    })
}

pub fn stakers(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();
    let reward = reward::read(deps.storage).unwrap();
    let start = start_after.map(|s| deps.api.addr_canonicalize(s.as_str()).unwrap());
    let users = user::batch_read(deps, start, limit)?;

    let mut stakers: Vec<resp::Staker> = Vec::new();
    for (address, user) in users.iter() {
        stakers.push(resp::Staker {
            address: address.to_string(),
            staked: user.amount,
            reward: util_staking::calculate_rewards(
                deps,
                &reward,
                user,
                config
                    .distribution_config
                    .applicable_reward_time(env.block.time.seconds()),
            )?,
        });
    }

    to_binary(&resp::StakersResponse { stakers })
}

pub fn balance_of(deps: Deps, _env: Env, owner: String) -> StdResult<Binary> {
    let user = user::read(
        deps.storage,
        &deps.api.addr_canonicalize(owner.as_str()).unwrap(),
    )
    .unwrap();

    to_binary(&resp::BalanceOfResponse {
        amount: user.amount,
    })
}

pub fn claimable_reward(deps: Deps, env: Env, owner: String) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();
    let reward = reward::read(deps.storage).unwrap();
    let user = user::read(
        deps.storage,
        &deps.api.addr_canonicalize(owner.as_str()).unwrap(),
    )
    .unwrap();

    to_binary(&resp::ClaimableRewardResponse {
        amount: util_staking::calculate_rewards(
            deps,
            &reward,
            &user,
            config
                .distribution_config
                .applicable_reward_time(env.block.time.seconds()),
        )?,
    })
}

pub fn available_cap_of(deps: Deps, _env: Env, address: String) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();
    let user = user::read(
        deps.storage,
        &deps.api.addr_canonicalize(address.as_str()).unwrap(),
    )
    .unwrap();

    if let Some(strategy) = config.cap_strategy {
        let resp: resp::AvailableCapOfResponse = deps.querier.query_wasm_smart(
            strategy,
            &QueryMsg::AvailableCapOf {
                address,
                amount: user.amount,
            },
        )?;
        to_binary(&resp)
    } else {
        to_binary(&resp::AvailableCapOfResponse {
            amount: None,
            unlimited: true,
        })
    }
}
