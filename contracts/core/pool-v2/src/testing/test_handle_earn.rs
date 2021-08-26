use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{Decimal, Uint128};
use cw20::TokenInfoResponse;
use pylon_core::pool_msg::HandleMsg;

use crate::contract;
use crate::testing::constants::{
    TEST_ADAPTER_INPUT_DENOM, TEST_BENEFICIARY, TEST_TOKEN_POOL, TEST_TOKEN_POOL_SUPPLY,
    TEST_TOKEN_YIELD, TEST_TOKEN_YIELD_SUPPLY,
};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::mock_tax::MockTax;
use crate::testing::mock_token::{balances_to_map, MockToken};
use crate::testing::utils;

#[test]
fn earn() {
    let mut deps = mock_dependencies(20, &[]);
    deps.querier.with_tax(MockTax::new(
        Decimal::percent(1),
        &[(
            &TEST_ADAPTER_INPUT_DENOM.to_string(),
            &Uint128::from(1000000u128),
        )],
    ));

    let mut mock_token = MockToken::default();
    mock_token.balances = balances_to_map(&[
        (
            &TEST_TOKEN_YIELD.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(TEST_TOKEN_YIELD_SUPPLY),
            )],
        ),
        (
            &TEST_TOKEN_POOL.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(TEST_TOKEN_POOL_SUPPLY),
            )],
        ),
    ]);
    deps.querier.with_token(mock_token);

    let _ = utils::initialize(&mut deps);
    let beneficiary = mock_env(TEST_BENEFICIARY, &[]);

    let msg = HandleMsg::Earn {};
    let res = contract::handle(&mut deps, beneficiary.clone(), msg).unwrap();
    println!("{:?}", res);
}
