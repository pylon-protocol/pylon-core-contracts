use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, StdResult, Storage,
};
use pylon_launchpad::lockup_resp as resp;

use crate::lib_staking as staking;
use crate::state;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: state::Config = state::read_config(&deps.storage)?;

    to_binary(&resp::ConfigResponse {
        owner: deps.api.human_address(&config.owner)?,
        share_token: deps.api.human_address(&config.share_token)?,
        reward_token: deps.api.human_address(&config.reward_token)?,
        start_time: config.start_time,
        cliff_time: config.cliff_time,
        finish_time: config.finish_time,
        temp_withdraw_start_time: config.temp_withdraw_start_time,
        temp_withdraw_finish_time: config.temp_withdraw_finish_time,
        reward_rate: config.reward_rate,
    })
}

pub fn reward<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let reward: state::Reward = state::read_reward(&deps.storage)?;

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
    let reward = state::read_reward(&deps.storage)?;
    let users = state::batch_read_user(deps, start_after, limit)?;

    let mut stakers: Vec<resp::Staker> = Vec::new();
    for (address, user) in users.iter() {
        stakers.push(resp::Staker {
            address: address.clone(),
            staked: user.amount,
            reward: staking::calculate_rewards(deps, &reward, &user, timestamp)?,
        });
    }

    to_binary(&resp::StakersResponse { stakers })
}

pub fn balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let user: state::User = state::read_user(&deps.storage, &deps.api.canonical_address(&owner)?)?;

    to_binary(&resp::BalanceOfResponse {
        amount: user.amount,
    })
}

pub fn claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    timestamp: Option<u64>,
) -> StdResult<Binary> {
    let reward: state::Reward = state::read_reward(&deps.storage)?;
    let user: state::User = state::read_user(&deps.storage, &deps.api.canonical_address(&owner)?)?;

    to_binary(&resp::ClaimableRewardResponse {
        amount: staking::calculate_rewards(deps, &reward, &user, timestamp)?,
    })
}
