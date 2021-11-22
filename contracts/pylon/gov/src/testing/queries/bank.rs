use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, Uint128};
use pylon_token::gov_msg::ClaimableAirdrop;
use pylon_token::gov_resp::StakerResponse;

use crate::queries::bank::query_staker;
use crate::testing::executions::{
    airdrop_instantiate, airdrop_update, instantiate, staking_deposit,
};
use crate::testing::{mock_deps, mock_env_height, TEST_TOKEN, TEST_VOTER, VOTING_TOKEN};

#[test]
fn stakers_filter_zero_reward() {
    let mut deps = mock_deps();
    instantiate::default(&mut deps);
    let (env, _, _) = airdrop_instantiate::default(&mut deps, TEST_TOKEN, 86400);

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
    )]);

    staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(100u128),
    )
    .unwrap();

    airdrop_update::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        Some(TEST_VOTER.to_string()),
    )
    .unwrap();

    let response = query_staker(deps.as_ref(), mock_env(), TEST_VOTER.to_string()).unwrap();
    let response: StakerResponse = from_binary(&response).unwrap();
    assert_eq!(response.claimable_airdrop, vec![]);

    let response = query_staker(
        deps.as_ref(),
        mock_env_height(env.block.height, env.block.time.seconds() + 86400 * 2),
        TEST_VOTER.to_string(),
    )
    .unwrap();
    let response: StakerResponse = from_binary(&response).unwrap();
    assert_eq!(
        response.claimable_airdrop,
        vec![(
            0,
            ClaimableAirdrop {
                token: TEST_TOKEN.to_string(),
                amount: Uint128::from(86400u128)
            }
        )]
    );
}
