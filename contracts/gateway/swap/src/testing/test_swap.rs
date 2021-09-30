use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, Coin, HumanAddr, Timestamp, Uint128};
use pylon_gateway::swap_msg::{ExecuteMsg, QueryMsg};
use pylon_gateway::swap_resp::AvailableCapOfResponse;
use pylon_token::gov::StakerResponse;

use crate::contract;
use crate::testing::constants::{TEST_MIN_STAKE_AMOUNT, TEST_OWNER, TEST_POOL_X_DENOM, TEST_USER};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::mock_staking::MockStaking;
use crate::testing::utils;

#[test]
fn sale() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);

    let mut user = mock_info(
        TEST_USER,
        &[Coin {
            denom: TEST_POOL_X_DENOM.to_string(),
            amount: Uint128::from(2000u128 * 1000000u128),
        }],
    );
    env.block.time = Timestamp::from_seconds(0);

    deps.querier.with_staking(MockStaking::new(&[(
        &user.message.sender.to_string(),
        StakerResponse {
            balance: Uint128::from(TEST_MIN_STAKE_AMOUNT),
            share: Uint128::from(TEST_MIN_STAKE_AMOUNT),
            locked_balance: vec![],
        },
    )]));

    let msg = ExecuteMsg::Deposit {};
    let res = contract::execute(deps.as_mut(), env.clone(), user, msg).unwrap();
    println!("{:?}", res);

    let mut owner = mock_info(TEST_OWNER, &[]);
    env.block.time = Timestamp::from_seconds(1);

    let msg = ExecuteMsg::Earn {};
    let res = contract::execute(deps.as_mut(), env, owner, msg).unwrap();
    println!("{:?}", res);
}

#[test]
fn calculate_user_cap() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);

    let user = HumanAddr::from(TEST_USER);
    let msg = QueryMsg::AvailableCapOf {
        address: TEST_USER.to_string(),
    };

    // < min_stake_amount
    deps.querier.with_staking(MockStaking::new(&[(
        &user.to_string(),
        StakerResponse {
            balance: Uint128::from(TEST_MIN_STAKE_AMOUNT / 2),
            share: Uint128::from(TEST_MIN_STAKE_AMOUNT / 2),
            locked_balance: vec![],
        },
    )]));
    let cap_res: AvailableCapOfResponse =
        from_binary(&contract::query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();
    assert_eq!(
        cap_res,
        AvailableCapOfResponse {
            staked: Uint256::from(TEST_MIN_STAKE_AMOUNT / 2),
            cap: Uint256::zero(),
        }
    );
}
