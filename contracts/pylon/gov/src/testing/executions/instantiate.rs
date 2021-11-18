use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Api, Decimal, Env, MessageInfo, Response, StdError, Uint128};
use pylon_token::gov_msg::InstantiateMsg;

use crate::error::ContractError;
use crate::executions::{instantiate, ExecuteResult};
use crate::state::config::Config;
use crate::state::state::State;
use crate::testing::{mock_deps, MockDeps, TEST_CREATOR, VOTING_TOKEN};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ExecuteResult {
    instantiate(deps.as_mut(), env, info, msg)
}

pub fn default(deps: &mut MockDeps) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = mock_info(TEST_CREATOR, &[]);

    let response = exec(deps, env.clone(), info.clone(), default_msg()).unwrap();

    (env, info, response)
}

pub fn default_msg() -> InstantiateMsg {
    InstantiateMsg {
        voting_token: VOTING_TOKEN.to_string(),
        quorum: Decimal::percent(30u64),
        threshold: Decimal::percent(50u64),
        voting_period: 20000u64,
        timelock_period: 10000u64,
        proposal_deposit: Uint128::from(10000000000u128),
        snapshot_period: 10u64,
    }
}

#[test]
fn success() {
    let mut deps = mock_deps();

    let (_, _, response) = default(&mut deps);
    assert_eq!(0, response.messages.len());

    let default_msg = default_msg();
    let config = Config::load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            pylon_token: deps.api.addr_canonicalize(VOTING_TOKEN).unwrap(),
            owner: deps.api.addr_canonicalize(TEST_CREATOR).unwrap(),
            quorum: default_msg.quorum,
            threshold: default_msg.threshold,
            voting_period: default_msg.voting_period,
            timelock_period: default_msg.timelock_period,
            expiration_period: 0u64, // Deprecated
            proposal_deposit: default_msg.proposal_deposit,
            snapshot_period: default_msg.snapshot_period
        }
    );

    let state = State::load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        State {
            poll_count: 0,
            total_share: Uint128::zero(),
            total_deposit: Uint128::zero(),
            total_airdrop_count: 0,
            airdrop_update_candidates: vec![]
        }
    );
}

#[test]
fn fail_invalid_quorum() {
    let mut deps = mock_deps();
    let mut msg = default_msg();
    msg.quorum = Decimal::percent(101);

    match exec(&mut deps, mock_env(), mock_info(TEST_CREATOR, &[]), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "quorum must be 0 to 1")
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fail_invalid_threshold() {
    let mut deps = mock_deps();
    let mut msg = default_msg();
    msg.threshold = Decimal::percent(101);

    match exec(&mut deps, mock_env(), mock_info(TEST_CREATOR, &[]), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "threshold must be 0 to 1")
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
