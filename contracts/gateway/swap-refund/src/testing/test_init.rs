use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{HumanAddr, InitResponse};
use std::str::FromStr;

use crate::contract;
use crate::state::config;
use crate::testing::constants::{TEST_BASE_PRICE, TEST_MANAGER, TEST_OWNER, TEST_REFUND_DENOM};
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = utils::init_msg();
    let env = mock_env(TEST_OWNER, &[]);
    let res = contract::init(&mut deps, env, msg).unwrap();
    assert_eq!(res, InitResponse::default());

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            manager: HumanAddr::from(TEST_MANAGER),
            refund_denom: TEST_REFUND_DENOM.to_string(),
            base_price: Decimal256::from_str(TEST_BASE_PRICE).unwrap(),
        }
    );
}
