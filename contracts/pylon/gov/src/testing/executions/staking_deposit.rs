use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Deps, Env, MessageInfo, Response, Uint128};

use crate::error::ContractError;
use crate::executions::staking::stake_voting_tokens;
use crate::executions::ExecuteResult;
use crate::state::state::State;
use crate::testing::{mock_deps, MockDeps, TEST_VOTER, VOTING_TOKEN};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> ExecuteResult {
    stake_voting_tokens(deps.as_mut(), env, info, sender, amount)
}

pub fn assert_stake_tokens_result(
    total_share: u128,
    total_deposit: u128,
    new_share: u128,
    poll_count: u64,
    execute_res: Response,
    deps: Deps,
) {
    assert_eq!(
        execute_res.attributes.get(2).expect("no log"),
        &attr("share", new_share.to_string())
    );

    let state = State::load(deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            poll_count,
            total_share: Uint128::from(total_share),
            total_deposit: Uint128::from(total_deposit),
            total_airdrop_count: 0,
            airdrop_update_candidates: vec![]
        }
    );
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    let response = exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(11u128),
    )
    .unwrap();
    assert_stake_tokens_result(11, 0, 11, 0, response, deps.as_ref());
}

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(11u128),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fail_insufficient_funds() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(0u128),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InsufficientFunds {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
