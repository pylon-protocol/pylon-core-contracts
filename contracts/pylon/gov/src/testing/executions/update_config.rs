use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, Decimal, Env, MessageInfo, Uint128};

use pylon_token::gov_resp::ConfigResponse;

use crate::error::ContractError;
use crate::executions::{update_config, ExecuteResult};
use crate::queries::config::query_config;
use crate::testing::{mock_deps, MockDeps, TEST_CREATOR, TEST_VOTER};

#[derive(Clone)]
pub struct Message {
    pub owner: Option<String>,
    pub quorum: Option<Decimal>,
    pub threshold: Option<Decimal>,
    pub voting_period: Option<u64>,
    pub timelock_period: Option<u64>,
    pub proposal_deposit: Option<Uint128>,
    pub snapshot_period: Option<u64>,
}

pub fn exec(deps: &mut MockDeps, _env: Env, info: MessageInfo, msg: Message) -> ExecuteResult {
    update_config(
        deps.as_mut(),
        info,
        msg.owner,
        msg.quorum,
        msg.threshold,
        msg.voting_period,
        msg.timelock_period,
        msg.proposal_deposit,
        msg.snapshot_period,
    )
}

pub fn default_msg() -> Message {
    Message {
        owner: None,
        quorum: None,
        threshold: None,
        voting_period: None,
        timelock_period: None,
        proposal_deposit: None,
        snapshot_period: None,
    }
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    // try update owner
    let mut msg = default_msg();
    msg.owner = Some(TEST_VOTER.to_string());
    exec(&mut deps, mock_env(), mock_info(TEST_CREATOR, &[]), msg).unwrap();

    let response = query_config(deps.as_ref()).unwrap();
    let response: ConfigResponse = from_binary(&response).unwrap();
    assert_eq!(response.owner, TEST_VOTER.to_string());

    // try update others
    let mut msg = default_msg();
    msg.quorum = Some(Decimal::percent(20));
    msg.threshold = Some(Decimal::percent(75));
    msg.voting_period = Some(20000u64);
    msg.timelock_period = Some(20000u64);
    msg.proposal_deposit = Some(Uint128::from(123u128));
    msg.snapshot_period = Some(11);
    exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        msg.clone(),
    )
    .unwrap();

    let response = query_config(deps.as_ref()).unwrap();
    let response: ConfigResponse = from_binary(&response).unwrap();
    assert_eq!(response.quorum, msg.quorum.unwrap());
    assert_eq!(response.threshold, msg.threshold.unwrap());
    assert_eq!(response.voting_period, msg.voting_period.unwrap());
    assert_eq!(response.timelock_period, msg.timelock_period.unwrap());
    assert_eq!(response.proposal_deposit, msg.proposal_deposit.unwrap());
    assert_eq!(response.snapshot_period, msg.snapshot_period.unwrap());
}

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        default_msg(),
    ) {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}
