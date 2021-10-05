use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::Response;
use pylon_gateway::swap_msg::Strategy;
use std::str::FromStr;

use crate::contract;
use crate::state::{config, state};
use crate::testing::constants::*;
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
            start: 1,
            finish: 11,
            cap_strategy: None,
            distribution_strategy: vec![
                Strategy::Lockup {
                    release_time: 5,
                    release_amount: Decimal256::percent(TEST_STRATEGY_LOCKUP_PERCENT),
                },
                Strategy::Vesting {
                    release_start_time: 5,
                    release_finish_time: 11,
                    release_amount: Decimal256::percent(TEST_STRATEGY_VESTING_PERCENT),
                }
            ],
            whitelist_enabled: true,
            price: Decimal256::from_str(TEST_PRICE).unwrap(),
            swap_pool_size: Uint256::from(TEST_SWAP_POOL_SIZE)
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
