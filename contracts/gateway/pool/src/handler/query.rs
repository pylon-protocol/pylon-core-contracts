use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, StdResult, Storage,
};
use pylon_gateway::pool_resp as resp;

use crate::handler::util_staking;
use crate::state::{config, reward, user};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = config::read(&deps.storage).unwrap();

    to_binary(&config) // TODO: marshal config
}

pub fn reward<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let reward = reward::read(&deps.storage).unwrap();

    to_binary(&resp::RewardResponse {
        total_deposit: reward.total_deposit,
        last_update_time: reward.last_update_time,
    })
}

pub fn stakers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
    timestamp: Option<u64>,
) -> StdResult<Binary> {
    let config = config::read(&deps.storage).unwrap();
    let reward = reward::read(&deps.storage).unwrap();
    let users = user::batch_read(deps, start_after, limit)?;

    let mut stakers: Vec<resp::Staker> = Vec::new();
    for (address, user) in users.iter() {
        stakers.push(resp::Staker {
            address: address.clone(),
            staked: user.amount,
            reward: util_staking::calculate_rewards(
                deps,
                &reward,
                &user,
                timestamp.map(|t| config.distribution_config.applicable_reward_time(t)),
            )?,
        });
    }

    to_binary(&resp::StakersResponse { stakers })
}

pub fn balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let user = user::read(&deps.storage, &deps.api.canonical_address(&owner).unwrap()).unwrap();

    to_binary(&resp::BalanceOfResponse {
        amount: user.amount,
    })
}

pub fn claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    timestamp: Option<u64>,
) -> StdResult<Binary> {
    let config = config::read(&deps.storage).unwrap();
    let reward = reward::read(&deps.storage).unwrap();
    let user = user::read(&deps.storage, &deps.api.canonical_address(&owner).unwrap()).unwrap();

    to_binary(&resp::ClaimableRewardResponse {
        amount: util_staking::calculate_rewards(
            deps,
            &reward,
            &user,
            timestamp.map(|t| config.distribution_config.applicable_reward_time(t)),
        )?,
    })
}
