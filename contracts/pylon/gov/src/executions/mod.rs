use cosmwasm_std::{
    from_binary, to_binary, CanonicalAddr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Order,
    Response, Uint128, WasmMsg,
};
use cosmwasm_storage::{ReadonlyBucket, ReadonlySingleton};
use cw20::Cw20ReceiveMsg;
use pylon_token::gov_msg::{
    AirdropMsg, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, StakingMsg,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::state::config::Config;
use crate::state::poll::{ExecuteData, Poll, PollCategory, PollStatus};
use crate::state::state::State;

pub type ExecuteResult = Result<Response, ContractError>;

pub mod airdrop;
pub mod poll;
pub mod staking;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    let response = Response::default().add_attribute("action", "instantiate");

    let config = Config {
        pylon_token: deps.api.addr_canonicalize(msg.voting_token.as_str())?,
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        timelock_period: msg.timelock_period,
        expiration_period: 0u64, // Deprecated
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
    };
    config.validate()?;

    let state = State {
        poll_count: 0,
        total_share: Uint128::zero(),
        total_deposit: Uint128::zero(),
        total_airdrop_count: 0,
        airdrop_update_candidates: vec![],
    };

    Config::save(deps.storage, &config)?;
    State::save(deps.storage, &state)?;

    Ok(response)
}

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> ExecuteResult {
    // only asset contract can execute this message
    let config = Config::load(deps.storage)?;
    if config.pylon_token != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Stake {}) => Ok(Response::new()
            // 1. Update reward
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Airdrop(AirdropMsg::Update {
                    target: Some(cw20_msg.sender.to_string()),
                }))?,
                funds: vec![],
            }))
            // 2. Execute Stake
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                msg: to_binary(&ExecuteMsg::Staking(StakingMsg::StakeInternal {
                    sender: cw20_msg.sender.to_string(),
                    amount: cw20_msg.amount,
                }))?,
                funds: vec![],
            }))),
        Ok(Cw20HookMsg::CreatePoll {
            title,
            category,
            description,
            link,
            execute_msgs,
        }) => poll::create(
            deps,
            env,
            cw20_msg.sender,
            cw20_msg.amount,
            title,
            category.into(),
            description,
            link,
            execute_msgs,
        ),
        _ => Err(ContractError::DataShouldBeGiven {}),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    timelock_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "update_config");

    let api = deps.api;
    let mut config = Config::load(deps.storage)?;

    if config.owner != api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(owner) = owner {
        config.owner = api.addr_canonicalize(&owner)?;
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

    if let Some(proposal_deposit) = proposal_deposit {
        config.proposal_deposit = proposal_deposit;
    }

    if let Some(period) = snapshot_period {
        config.snapshot_period = period;
    }

    Config::save(deps.storage, &config)?;

    Ok(response)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyState {
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LegacyPoll {
    pub id: u64,
    pub creator: CanonicalAddr,
    pub status: PollStatus,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub end_height: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub execute_data: Option<Vec<ExecuteData>>,
    pub deposit_amount: Uint128,
    /// Total balance at the end poll
    pub total_balance_at_end_poll: Option<Uint128>,
    pub staked_amount: Option<Uint128>,
}

pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> ExecuteResult {
    match msg {
        MigrateMsg::State {} => {
            let state: LegacyState = ReadonlySingleton::new(deps.storage, b"state")
                .load()
                .unwrap();

            State::save(
                deps.storage,
                &State {
                    poll_count: state.poll_count,
                    total_share: state.total_share,
                    total_deposit: state.total_deposit,
                    total_airdrop_count: 0,
                    airdrop_update_candidates: vec![],
                },
            )
            .unwrap();

            let legacy_poll_store: ReadonlyBucket<LegacyPoll> =
                ReadonlyBucket::new(deps.storage, b"poll");
            let legacy_polls: Vec<LegacyPoll> = legacy_poll_store
                .range(None, None, Order::Descending)
                .take(100)
                .map(|item| -> LegacyPoll {
                    let (_, v) = item.unwrap();
                    v
                })
                .collect();

            for poll in legacy_polls.iter() {
                Poll::save(
                    deps.storage,
                    &poll.id,
                    &Poll {
                        id: poll.id,
                        creator: poll.creator.clone(),
                        status: poll.status.clone(),
                        yes_votes: poll.yes_votes,
                        no_votes: poll.no_votes,
                        end_height: poll.end_height,
                        title: poll.title.clone(),
                        category: PollCategory::None,
                        description: poll.description.clone(),
                        link: poll.link.clone(),
                        execute_data: poll.execute_data.clone(),
                        deposit_amount: poll.deposit_amount,
                        total_balance_at_end_poll: poll.total_balance_at_end_poll,
                        staked_amount: poll.staked_amount,
                    },
                )?;
            }
        }
        MigrateMsg::General {} => {}
    }

    Ok(Response::default())
}
