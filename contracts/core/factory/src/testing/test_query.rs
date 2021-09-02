use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{from_binary, HumanAddr};
use pylon_core::factory_msg::{HandleMsg, QueryMsg};
use pylon_core::factory_resp;
use std::str::FromStr;

use crate::contract;
use crate::state::pool;
use crate::testing::constants::{
    TEST_CREATOR, TEST_FEE_COLLECTOR, TEST_FEE_RATE, TEST_POOL, TEST_POOL_CODE_ID,
    TEST_POOL_REWARD_AMOUNT, TEST_POOL_REWARD_FEE, TEST_POOL_TOKEN, TEST_POOL_TOKEN_SUPPLY,
    TEST_TOKEN_CODE_ID, TEST_YIELD_ADAPTER, TEST_YIELD_TOKEN, TEST_YIELD_TOKEN_SUPPLY,
};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn query_config() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let msg = QueryMsg::Config {};
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: factory_resp::ConfigResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::ConfigResponse {
            owner: HumanAddr::from(TEST_CREATOR),
            pool_code_id: TEST_POOL_CODE_ID,
            token_code_id: TEST_TOKEN_CODE_ID,
            fee_rate: Decimal256::from_str(TEST_FEE_RATE).unwrap(),
            fee_collector: HumanAddr::from(TEST_FEE_COLLECTOR),
        }
    )
}

#[test]
fn query_pool_info() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    for x in 0..5 {
        pool::store(
            &mut deps.storage,
            x,
            &pool::Pool {
                id: x,
                status: pool::Status::Deployed,
                address: Default::default(),
            },
        )
        .unwrap();
    }

    let msg = QueryMsg::PoolInfo { pool_id: 0 };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: factory_resp::PoolInfoResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::PoolInfoResponse {
            id: 0,
            address: HumanAddr::from(TEST_POOL),
            dp_address: HumanAddr::from(TEST_POOL_TOKEN),
            dp_total_supply: Uint256::from(TEST_POOL_TOKEN_SUPPLY),
            yield_adapter: HumanAddr::from(TEST_YIELD_ADAPTER),
            yield_token: HumanAddr::from(TEST_YIELD_TOKEN),
            yield_token_balance: Uint256::from(TEST_YIELD_TOKEN_SUPPLY),
            accumulated_reward: Uint256::from(TEST_POOL_REWARD_AMOUNT),
            accumulated_fee: Uint256::from(TEST_POOL_REWARD_FEE),
        }
    )
}
