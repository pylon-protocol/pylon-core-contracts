use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{Api, HumanAddr, InitResponse};
use std::str::FromStr;

use crate::contract;
use crate::state::{config, state};
use crate::testing::constants::{
    TEST_CREATOR, TEST_FEE_COLLECTOR, TEST_FEE_RATE, TEST_POOL_CODE_ID, TEST_TOKEN_CODE_ID,
};
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = utils::init_msg();
    let env = mock_env(TEST_CREATOR, &[]);
    let res = contract::init(&mut deps, env.clone(), msg).unwrap();
    assert_eq!(res, InitResponse::default());

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            owner: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_CREATOR))
                .unwrap(),
            pool_code_id: TEST_POOL_CODE_ID,
            token_code_id: TEST_TOKEN_CODE_ID,
            fee_rate: Decimal256::from_str(TEST_FEE_RATE).unwrap(),
            fee_collector: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_FEE_COLLECTOR))
                .unwrap()
        }
    );

    let state = state::read(&deps.storage).unwrap();
    assert_eq!(state, state::State { next_pool_id: 0 });
}
