use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::Response;
use pylon_core::test_constant::*;

use crate::contract;
use crate::state::{config, state};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let msg = utils::init_msg();
    let env = mock_env();
    let info = mock_info(TEST_CREATOR, &[]);
    let res = contract::instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res, Response::default());

    let config = config::read(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            owner: TEST_CREATOR.to_string(),
            pool_code_id: TEST_POOL_CODE_ID,
            token_code_id: TEST_TOKEN_CODE_ID,
            fee_rate: Decimal256::percent(TEST_FACTORY_FEE_RATE),
            fee_collector: TEST_FEE_COLLECTOR.to_string()
        }
    );

    let state = state::read(deps.as_ref().storage).unwrap();
    assert_eq!(state, state::State { next_pool_id: 0 });
}
