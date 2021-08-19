use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use pylon_gateway::common::OrderBy;
use pylon_gateway::gov::{
    ConfigResponse, ExecuteMsg, PollResponse, PollStatus, PollsResponse, StakerResponse,
    StateResponse, VotersResponse, VotersResponseItem,
};

use crate::querier::gov;
use crate::state::{bank, config, poll, state};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<ConfigResponse> {
    let config = config::read(&deps.storage).load()?;
    Ok(ConfigResponse {
        owner: deps.api.human_address(&config.owner)?,
        pylon_token: deps.api.human_address(&config.pylon_token)?,
        quorum: config.quorum,
        threshold: config.threshold,
        voting_period: config.voting_period,
        timelock_period: config.timelock_period,
        expiration_period: config.expiration_period,
        proposal_deposit: config.proposal_deposit,
        snapshot_period: config.snapshot_period,
    })
}

pub fn state<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<StateResponse> {
    let state = state::read(&deps.storage).load()?;
    Ok(StateResponse {
        poll_count: state.poll_count,
        total_share: state.total_share,
        total_deposit: state.total_deposit,
    })
}

pub fn poll<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    poll_id: u64,
) -> StdResult<PollResponse> {
    let poll = match poll::read(&deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(StdError::generic_err("Poll does not exist")),
    }
    .unwrap();

    let mut data_list: Vec<ExecuteMsg> = vec![];

    Ok(PollResponse {
        id: poll.id,
        creator: deps.api.human_address(&poll.creator).unwrap(),
        status: poll.status,
        end_height: poll.end_height,
        title: poll.title,
        description: poll.description,
        link: poll.link,
        deposit_amount: poll.deposit_amount,
        execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
            for msg in exe_msgs {
                let execute_data = ExecuteMsg {
                    order: msg.order,
                    contract: deps.api.human_address(&msg.contract)?,
                    msg: msg.msg,
                };
                data_list.push(execute_data)
            }
            Some(data_list)
        } else {
            None
        },
        yes_votes: poll.yes_votes,
        no_votes: poll.no_votes,
        staked_amount: poll.staked_amount,
        total_balance_at_end_poll: poll.total_balance_at_end_poll,
    })
}

pub fn polls<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<PollsResponse> {
    let polls = poll::read_polls(&deps.storage, filter, start_after, limit, order_by)?;

    let poll_responses: StdResult<Vec<PollResponse>> = polls
        .iter()
        .map(|poll| {
            Ok(PollResponse {
                id: poll.id,
                creator: deps.api.human_address(&poll.creator).unwrap(),
                status: poll.status.clone(),
                end_height: poll.end_height,
                title: poll.title.to_string(),
                description: poll.description.to_string(),
                link: poll.link.clone(),
                deposit_amount: poll.deposit_amount,
                execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
                    let mut data_list: Vec<ExecuteMsg> = vec![];

                    for msg in exe_msgs {
                        let execute_data = ExecuteMsg {
                            order: msg.order,
                            contract: deps.api.human_address(&msg.contract)?,
                            msg: msg.msg,
                        };
                        data_list.push(execute_data)
                    }
                    Some(data_list)
                } else {
                    None
                },
                yes_votes: poll.yes_votes,
                no_votes: poll.no_votes,
                staked_amount: poll.staked_amount,
                total_balance_at_end_poll: poll.total_balance_at_end_poll,
            })
        })
        .collect();

    Ok(PollsResponse {
        polls: poll_responses?,
    })
}

pub fn voters<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    poll_id: u64,
    start_after: Option<HumanAddr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<VotersResponse> {
    let poll = match poll::read(&deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(StdError::generic_err("Poll does not exist")),
    }
    .unwrap();

    let voters = if poll.status != PollStatus::InProgress {
        vec![]
    } else if let Some(start_after) = start_after {
        poll::read_poll_voters(
            &deps.storage,
            poll_id,
            Some(deps.api.canonical_address(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        poll::read_poll_voters(&deps.storage, poll_id, None, limit, order_by)?
    };

    let voters_response: StdResult<Vec<VotersResponseItem>> = voters
        .iter()
        .map(|voter_info| {
            Ok(VotersResponseItem {
                voter: deps.api.human_address(&voter_info.0)?,
                vote: voter_info.1.vote.clone(),
                balance: voter_info.1.balance,
            })
        })
        .collect();

    Ok(VotersResponse {
        voters: voters_response?,
    })
}

pub fn staker<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<StakerResponse> {
    let addr_raw = deps.api.canonical_address(&address).unwrap();
    let config = config::read(&deps.storage).load()?;
    let state = state::read(&deps.storage).load()?;
    let mut token_manager = bank::read(&deps.storage)
        .may_load(addr_raw.as_slice())?
        .unwrap_or_default();

    // filter out not in-progress polls
    token_manager.locked_balance.retain(|(poll_id, _)| {
        let poll = poll::read(&deps.storage)
            .load(&poll_id.to_be_bytes())
            .unwrap();

        poll.status == PollStatus::InProgress
    });

    let total_balance = (gov::load_token_balance(
        &deps,
        &deps.api.human_address(&config.pylon_token)?,
        &state.contract_addr,
    )? - state.total_deposit)?;

    Ok(StakerResponse {
        balance: if !state.total_share.is_zero() {
            token_manager
                .share
                .multiply_ratio(total_balance, state.total_share)
        } else {
            Uint128::zero()
        },
        share: token_manager.share,
        locked_balance: token_manager.locked_balance,
    })
}
