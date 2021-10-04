use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{Env, MessageInfo, OwnedDeps};
use pylon_gateway::swap_msg::{ConfigureMsg, ExecuteMsg, InstantiateMsg, Strategy};
use std::str::FromStr;

use crate::contract;
use crate::testing::constants::*;
use crate::testing::mock_querier::CustomMockQuerier;

pub fn init_msg() -> InstantiateMsg {
    InstantiateMsg {
        beneficiary: TEST_BENEFICIARY.to_string(),
        pool_x_denom: TEST_POOL_X_DENOM.to_string(),
        pool_y_addr: TEST_POOL_Y_ADDR.to_string(),
        pool_liq_x: Uint256::from(TEST_POOL_LIQ_X),
        pool_liq_y: Uint256::from(TEST_POOL_LIQ_Y),
        price: Decimal256::from_str(TEST_PRICE).unwrap(),
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
            },
        ],
        whitelist_enabled: true,
        start: 1,
        period: 10,
        swap_pool_size: Uint256::from(TEST_SWAP_POOL_SIZE),
    }
}

pub fn initialize(
    deps: &mut OwnedDeps<MockStorage, MockApi, CustomMockQuerier>,
) -> (Env, MessageInfo) {
    let env = mock_env();
    let info = mock_info(TEST_OWNER, &[]);
    let msg = init_msg();
    let _res = contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: contract initialized");

    let msg = ExecuteMsg::Configure(ConfigureMsg::Whitelist {
        whitelist: true,
        candidates: vec![TEST_OWNER.to_string(), TEST_USER_1.to_string()],
    });
    let _res = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: setup whitelist");

    (env, info)
}
