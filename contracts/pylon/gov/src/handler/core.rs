use cosmwasm_std::{
    from_binary, Api, CanonicalAddr, Decimal, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    Querier, StdError, Storage, Uint128,
};
use cw20::Cw20ReceiveMsg;
use pylon_token::gov::Cw20HookMsg;

use crate::handler::{poll, staker};
use crate::state::config;

pub fn register_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    pylon_token: HumanAddr,
) -> HandleResult {
    let mut config = config::read(&deps.storage).load()?;
    if config.pylon_token != CanonicalAddr::default() {
        return Err(StdError::unauthorized());
    }

    config.pylon_token = deps.api.canonical_address(&pylon_token)?;
    config::store(&mut deps.storage).save(&config)?;

    Ok(HandleResponse::default())
}

pub fn receive_cw20<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> HandleResult {
    // only asset contract can execute this message
    let config = config::read(&deps.storage).load()?;
    if config.pylon_token != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::unauthorized());
    }

    if let Some(msg) = cw20_msg.msg {
        match from_binary(&msg)? {
            Cw20HookMsg::StakeVotingTokens {} => {
                staker::stake_voting_tokens(deps, env, cw20_msg.sender, cw20_msg.amount)
            }
            Cw20HookMsg::CreatePoll {
                title,
                description,
                link,
                execute_msgs,
            } => poll::create(
                deps,
                env,
                cw20_msg.sender,
                cw20_msg.amount,
                title,
                description,
                link,
                execute_msgs,
            ),
        }
    } else {
        Err(StdError::generic_err("data should be given"))
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    timelock_period: Option<u64>,
    expiration_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> HandleResult {
    let api = deps.api;
    config::store(&mut deps.storage).update(|mut config| {
        if config.owner != api.canonical_address(&env.message.sender)? {
            return Err(StdError::unauthorized());
        }

        if let Some(owner) = owner {
            config.owner = api.canonical_address(&owner)?;
        }

        if let Some(quorum) = quorum {
            config.quorum = quorum;
        }

        if let Some(threshold) = threshold {
            config.threshold = threshold;
        }

        if let Some(voting_period) = voting_period {
            config.voting_period = voting_period;
        }

        if let Some(timelock_period) = timelock_period {
            config.timelock_period = timelock_period;
        }

        if let Some(expiration_period) = expiration_period {
            config.expiration_period = expiration_period;
        }

        if let Some(proposal_deposit) = proposal_deposit {
            config.proposal_deposit = proposal_deposit;
        }

        if let Some(period) = snapshot_period {
            config.snapshot_period = period;
        }

        Ok(config)
    })?;
    Ok(HandleResponse::default())
}
