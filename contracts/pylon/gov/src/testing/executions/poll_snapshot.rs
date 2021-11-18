use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Env, MessageInfo, Uint128};

use crate::error::ContractError;
use crate::executions::poll::snapshot;
use crate::executions::ExecuteResult;
use crate::testing::{mock_deps, mock_env_height, MockDeps, TEST_CREATOR, VOTING_TOKEN};

pub fn exec(deps: &mut MockDeps, env: Env, info: MessageInfo, poll_id: u64) -> ExecuteResult {
    snapshot(deps.as_mut(), env, info, poll_id)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_msg = super::instantiate::default_msg();

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + default_msg.proposal_deposit.u128()),
        )],
    )]);

    let (env, _, _) = super::poll_create::default(&mut deps);
    let response = exec(
        &mut deps,
        mock_env_height(env.block.height + default_msg.voting_period - 10, 0),
        mock_info(TEST_CREATOR, &[]),
        1,
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "snapshot_poll"),
            attr("poll_id", "1"),
            attr("staked_amount", 11u128.to_string()),
        ]
    );
}

#[test]
fn fail_height() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_msg = super::instantiate::default_msg();

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + default_msg.proposal_deposit.u128()),
        )],
    )]);

    super::poll_create::default(&mut deps);

    match exec(&mut deps, mock_env(), mock_info(TEST_CREATOR, &[]), 1) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::SnapshotHeight {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fail_already_occurred() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_msg = super::instantiate::default_msg();

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + default_msg.proposal_deposit.u128()),
        )],
    )]);

    let (env, _, _) = super::poll_create::default(&mut deps);
    exec(
        &mut deps,
        mock_env_height(env.block.height + default_msg.voting_period - 10, 0),
        mock_info(TEST_CREATOR, &[]),
        1,
    )
    .unwrap();

    match exec(
        &mut deps,
        mock_env_height(env.block.height + default_msg.voting_period - 5, 0),
        mock_info(TEST_CREATOR, &[]),
        1,
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::SnapshotAlreadyOccurred {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
