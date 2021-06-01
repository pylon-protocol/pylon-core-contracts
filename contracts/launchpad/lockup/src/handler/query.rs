use cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage};

use crate::lib_staking as staking;
use crate::resp;
use crate::state;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config: state::Config = state::read_config(&deps.storage)?;

    Ok(to_binary(&resp::ConfigResponse {
        owner: deps.api.human_address(&config.owner)?,
        share_token: deps.api.human_address(&config.share_token)?,
        reward_token: deps.api.human_address(&config.reward_token)?,
        start_time: config.start_time,
        finish_time: config.finish_time,
        open_deposit: config.open_deposit,
        open_withdraw: config.open_withdraw,
        open_claim: config.open_claim,
        reward_rate: config.reward_rate,
    })?)
}

pub fn reward<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let reward: state::Reward = state::read_reward(&deps.storage)?;

    Ok(to_binary(&resp::RewardResponse {
        total_deposit: reward.total_deposit,
        last_update_time: reward.last_update_time,
    })?)
}

pub fn balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let user: state::User = state::read_user(&deps.storage, &deps.api.canonical_address(&owner)?)?;

    Ok(to_binary(&resp::BalanceOfResponse {
        amount: user.amount,
    })?)
}

pub fn claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    timestamp: Option<u64>,
) -> StdResult<Binary> {
    let reward: state::Reward = state::read_reward(&deps.storage)?;
    let user: state::User = state::read_user(&deps.storage, &deps.api.canonical_address(&owner)?)?;

    Ok(to_binary(&resp::ClaimableRewardResponse {
        amount: staking::calculate_rewards(deps, &reward, &user, timestamp)?,
    })?)
}
