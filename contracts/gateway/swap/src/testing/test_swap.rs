use cosmwasm_std::testing::mock_info;
use cosmwasm_std::{Coin, Timestamp, Uint128};
use pylon_gateway::swap_msg::ExecuteMsg;

use crate::contract;
use crate::testing::constants::{TEST_OWNER, TEST_POOL_X_DENOM, TEST_USER};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn sale() {
    let mut deps = mock_dependencies(&[]);
    let (mut env, _) = utils::initialize(&mut deps);

    let user = mock_info(
        TEST_USER,
        &[Coin {
            denom: TEST_POOL_X_DENOM.to_string(),
            amount: Uint128::from(2000u128 * 1000000u128),
        }],
    );
    env.block.time = Timestamp::from_seconds(0);

    let msg = ExecuteMsg::Deposit {};
    let res = contract::execute(deps.as_mut(), env.clone(), user, msg).unwrap();
    println!("{:?}", res);

    let owner = mock_info(TEST_OWNER, &[]);
    env.block.time = Timestamp::from_seconds(1);

    let msg = ExecuteMsg::Earn {};
    let res = contract::execute(deps.as_mut(), env, owner, msg).unwrap();
    println!("{:?}", res);
}
