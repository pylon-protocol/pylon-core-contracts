use cosmwasm_std::{to_binary, Deps, StdResult};
use pylon_token::common::OrderBy;
use pylon_token::gov_msg::PollExecuteMsg;
use pylon_token::gov_resp::{PollResponse, PollsResponse, VotersResponse, VotersResponseItem};

use crate::error::ContractError;
use crate::queries::QueryResult;
use crate::state::poll::{Poll, PollCategory, PollStatus, VoterInfo};

pub fn query_poll(deps: Deps, poll_id: u64) -> QueryResult {
    let poll = match Poll::may_load(deps.storage, &poll_id)? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    Ok(to_binary(&to_response(deps, &poll)?)?)
}

pub fn query_polls(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> QueryResult {
    let polls = Poll::load_range(deps.storage, start_after, limit, order_by)?;

    let poll_responses: Vec<PollResponse> = polls
        .iter()
        .map(|poll| to_response(deps, poll).unwrap())
        .collect();

    Ok(to_binary(&PollsResponse {
        polls: poll_responses,
    })?)
}

pub fn query_polls_with_status_filter(
    deps: Deps,
    status_filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> QueryResult {
    let polls = Poll::load_range_with_status_filter(
        deps.storage,
        status_filter.unwrap_or(PollStatus::InProgress),
        start_after,
        limit,
        order_by,
    )?;

    let poll_responses: Vec<PollResponse> = polls
        .iter()
        .map(|poll| to_response(deps, poll).unwrap())
        .collect();

    Ok(to_binary(&PollsResponse {
        polls: poll_responses,
    })?)
}

pub fn query_polls_with_category_filter(
    deps: Deps,
    category_filter: Option<PollCategory>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> QueryResult {
    let polls = Poll::load_range_with_category_filter(
        deps.storage,
        category_filter.unwrap_or(PollCategory::None),
        start_after,
        limit,
        order_by,
    )?;

    let poll_responses: Vec<PollResponse> = polls
        .iter()
        .map(|poll| to_response(deps, poll).unwrap())
        .collect();

    Ok(to_binary(&PollsResponse {
        polls: poll_responses,
    })?)
}

pub fn query_voters(
    deps: Deps,
    poll_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> QueryResult {
    let poll = match Poll::may_load(deps.storage, &poll_id)? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    let voters = if poll.status != PollStatus::InProgress {
        vec![]
    } else if let Some(start_after) = start_after {
        VoterInfo::load_range(
            deps.storage,
            poll_id,
            Some(deps.api.addr_canonicalize(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        VoterInfo::load_range(deps.storage, poll_id, None, limit, order_by)?
    };

    let voters_response: StdResult<Vec<VotersResponseItem>> = voters
        .iter()
        .map(|voter_info| {
            Ok(VotersResponseItem {
                voter: deps.api.addr_humanize(&voter_info.0)?.to_string(),
                vote: voter_info.1.vote.clone().into(),
                balance: voter_info.1.balance,
            })
        })
        .collect();

    Ok(to_binary(&VotersResponse {
        voters: voters_response?,
    })?)
}

fn to_response(deps: Deps, poll: &Poll) -> StdResult<PollResponse> {
    Ok(PollResponse {
        id: poll.id,
        creator: deps.api.addr_humanize(&poll.creator)?.to_string(),
        status: poll.status.clone().into(),
        end_height: poll.end_height,
        title: poll.title.to_string(),
        category: poll.category.clone().into(),
        description: poll.description.to_string(),
        link: poll.link.clone(),
        deposit_amount: poll.deposit_amount,
        execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
            let mut data_list: Vec<PollExecuteMsg> = vec![];

            for msg in exe_msgs {
                let execute_data = PollExecuteMsg {
                    order: msg.order,
                    contract: deps.api.addr_humanize(&msg.contract)?.to_string(),
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
