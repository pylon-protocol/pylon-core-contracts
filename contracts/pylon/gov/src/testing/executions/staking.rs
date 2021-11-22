use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, from_binary, Uint128};
use pylon_token::gov_resp::StakerResponse;

use crate::queries::bank::query_staker;
use crate::testing::{mock_deps, TEST_VOTER, VOTING_TOKEN};

#[test]
fn share_calculation() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    // create 100 share
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
    )]);

    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(100u128),
    )
    .unwrap();

    // add more balance(100) to make share:balance = 1:2
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(200u128 + 100u128),
        )],
    )]);

    let response = super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(100u128),
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "staking"),
            attr("sender", TEST_VOTER),
            attr("share", "50"),
            attr("amount", "100"),
        ]
    );

    let response = super::staking_withdraw::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Some(Uint128::from(100u128)),
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "withdraw"),
            attr("recipient", TEST_VOTER),
            attr("amount", "100"),
        ]
    );

    // 100 tokens withdrawn
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(200u128))],
    )]);

    let response = query_staker(deps.as_ref(), mock_env(), TEST_VOTER.to_string()).unwrap();
    let stake_info: StakerResponse = from_binary(&response).unwrap();
    assert_eq!(stake_info.share, Uint128::new(100));
    assert_eq!(stake_info.balance, Uint128::new(200));
    assert_eq!(stake_info.locked_balance, vec![]);
}
