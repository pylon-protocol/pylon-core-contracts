use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{from_binary, Coin, HumanAddr, Uint128};
use pylon_gateway::swap_msg::{HandleMsg, QueryMsg};
use pylon_gateway::swap_resp::AvailableCapOfResponse;
use pylon_token::gov::StakerResponse;
use std::ops::{Add, Mul};
use std::str::FromStr;

use crate::contract;
use crate::testing::constants::{
    TEST_ADDITIONAL_CAP_PER_TOKEN, TEST_MAX_STAKE_AMOUNT, TEST_MAX_USER_CAP, TEST_MIN_STAKE_AMOUNT,
    TEST_MIN_USER_CAP, TEST_OWNER, TEST_POOL_X_DENOM, TEST_USER,
};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::mock_staking::MockStaking;
use crate::testing::utils;

#[test]
fn sale() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let mut user = mock_env(
        TEST_USER,
        &[Coin {
            denom: TEST_POOL_X_DENOM.to_string(),
            amount: Uint128::from(2000u128 * 1000000u128),
        }],
    );
    user.block.time = 0;

    deps.querier.with_staking(MockStaking::new(&[(
        &user.message.sender.to_string(),
        StakerResponse {
            balance: Uint128::from(TEST_MIN_STAKE_AMOUNT),
            share: Uint128::from(TEST_MIN_STAKE_AMOUNT),
            locked_balance: vec![],
        },
    )]));

    let msg = HandleMsg::Deposit {};
    let res = contract::handle(&mut deps, user.clone(), msg).unwrap();
    println!("{:?}", res);

    let mut owner = mock_env(TEST_OWNER, &[]);
    owner.block.time = 1;

    let msg = HandleMsg::Earn {};
    let res = contract::handle(&mut deps, owner.clone(), msg).unwrap();
    println!("{:?}", res);
}

#[test]
fn calculate_user_cap() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let user = HumanAddr::from(TEST_USER);
    let msg = QueryMsg::AvailableCapOf {
        address: HumanAddr::from(TEST_USER),
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
        from_binary(&contract::query(&deps, msg.clone()).unwrap()).unwrap();
    assert_eq!(
        cap_res,
        AvailableCapOfResponse {
            staked: Uint256::from(TEST_MIN_STAKE_AMOUNT / 2),
            cap: Uint256::from(TEST_MIN_USER_CAP),
        }
    );

    // == min_stake_amount
    deps.querier.with_staking(MockStaking::new(&[(
        &user.to_string(),
        StakerResponse {
            balance: Uint128::from(TEST_MIN_STAKE_AMOUNT),
            share: Uint128::from(TEST_MIN_STAKE_AMOUNT),
            locked_balance: vec![],
        },
    )]));
    let cap_res: AvailableCapOfResponse =
        from_binary(&contract::query(&deps, msg.clone()).unwrap()).unwrap();
    assert_eq!(
        cap_res,
        AvailableCapOfResponse {
            staked: Uint256::from(TEST_MIN_STAKE_AMOUNT),
            cap: Uint256::from(TEST_MIN_USER_CAP),
        }
    );

    // min_stake_amount < x < max_stake_amount
    deps.querier.with_staking(MockStaking::new(&[(
        &user.to_string(),
        StakerResponse {
            balance: Uint128::from(TEST_MIN_STAKE_AMOUNT * 2),
            share: Uint128::from(TEST_MIN_STAKE_AMOUNT * 2),
            locked_balance: vec![],
        },
    )]));
    let cap_res: AvailableCapOfResponse =
        from_binary(&contract::query(&deps, msg.clone()).unwrap()).unwrap();
    assert_eq!(
        cap_res,
        AvailableCapOfResponse {
            staked: Uint256::from(TEST_MIN_STAKE_AMOUNT * 2),
            cap: Uint256::from(TEST_MIN_USER_CAP).add(
                Uint256::from(TEST_MIN_STAKE_AMOUNT)
                    .mul(Decimal256::from_str(TEST_ADDITIONAL_CAP_PER_TOKEN).unwrap())
            ),
        }
    );

    // == max_stake_amount
    deps.querier.with_staking(MockStaking::new(&[(
        &user.to_string(),
        StakerResponse {
            balance: Uint128::from(TEST_MAX_STAKE_AMOUNT),
            share: Uint128::from(TEST_MAX_STAKE_AMOUNT),
            locked_balance: vec![],
        },
    )]));
    let cap_res: AvailableCapOfResponse =
        from_binary(&contract::query(&deps, msg.clone()).unwrap()).unwrap();
    assert_eq!(
        cap_res,
        AvailableCapOfResponse {
            staked: Uint256::from(TEST_MAX_STAKE_AMOUNT),
            cap: Uint256::from(TEST_MAX_USER_CAP)
        }
    );

    // > max_stake_amount
    deps.querier.with_staking(MockStaking::new(&[(
        &user.to_string(),
        StakerResponse {
            balance: Uint128::from(TEST_MAX_STAKE_AMOUNT * 2),
            share: Uint128::from(TEST_MAX_STAKE_AMOUNT * 2),
            locked_balance: vec![],
        },
    )]));
    let cap_res: AvailableCapOfResponse =
        from_binary(&contract::query(&deps, msg.clone()).unwrap()).unwrap();
    assert_eq!(
        cap_res,
        AvailableCapOfResponse {
            staked: Uint256::from(TEST_MAX_STAKE_AMOUNT * 2),
            cap: Uint256::from(TEST_MAX_USER_CAP)
        }
    );
}
