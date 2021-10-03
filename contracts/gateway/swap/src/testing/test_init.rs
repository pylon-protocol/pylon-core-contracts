use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::Response;
use std::str::FromStr;

use crate::contract;
use crate::state::{config, state};
use crate::testing::constants::{
    TEST_BASE_PRICE, TEST_BENEFICIARY, TEST_OWNER, TEST_POOL_LIQ_X, TEST_POOL_LIQ_Y,
    TEST_POOL_X_DENOM, TEST_POOL_Y_ADDR, TEST_TOTAL_SALE_AMOUNT,
};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = utils::init_msg();
    let env = mock_env();
    let info = mock_info(TEST_OWNER, &[]);
    let res = contract::instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res, Response::default());

    let config = config::read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        config::Config {
            owner: TEST_OWNER.to_string(),
            beneficiary: TEST_BENEFICIARY.to_string(),
            start: 0,
            finish: 1,
            cap_strategy: None,
            distribution_strategy: vec![],
            whitelist_enabled: false,
            price: Decimal256::from_str(TEST_BASE_PRICE).unwrap(),
            swap_pool_size: Uint256::from(TEST_TOTAL_SALE_AMOUNT)
        }
    );

    let state = state::read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        state::State {
            total_swapped: Uint256::zero(),
            total_claimed: Uint256::zero(),
            x_denom: TEST_POOL_X_DENOM.to_string(),
            y_addr: TEST_POOL_Y_ADDR.to_string(),
            liq_x: Uint256::from(TEST_POOL_LIQ_X),
            liq_y: Uint256::from(TEST_POOL_LIQ_Y)
        }
    );
}
