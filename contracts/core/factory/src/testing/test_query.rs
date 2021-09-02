use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{from_binary, Api, HumanAddr, Uint128};
use pylon_core::factory_msg::{HandleMsg, QueryMsg};
use pylon_core::factory_resp;
use std::str::FromStr;

use crate::contract;
use crate::state::pool;
use crate::testing::constants::{
    TEST_CREATOR, TEST_FEE_COLLECTOR, TEST_FEE_RATE, TEST_INPUT_DENOM, TEST_POOL,
    TEST_POOL_CODE_ID, TEST_POOL_REWARD_AMOUNT, TEST_POOL_REWARD_FEE, TEST_POOL_TOKEN,
    TEST_POOL_TOKEN_SUPPLY, TEST_TOKEN_CODE_ID, TEST_YIELD_ADAPTER, TEST_YIELD_TOKEN,
    TEST_YIELD_TOKEN_SUPPLY,
};
use crate::testing::mock_adapter::MockAdapter;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::mock_token::{balances_to_map, MockToken};
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

    let mut mock_token = MockToken::default();
    mock_token.balances = balances_to_map(&[(
        &TEST_YIELD_TOKEN.to_string(),
        &[(
            &TEST_POOL.to_string(),
            &Uint128::from(TEST_YIELD_TOKEN_SUPPLY),
        )],
    )]);
    deps.querier.with_token(mock_token);

    for x in 0..20 {
        pool::store(
            &mut deps.storage,
            x,
            &pool::Pool {
                id: x,
                status: pool::Status::Deployed,
                address: deps
                    .api
                    .canonical_address(&HumanAddr::from(TEST_POOL))
                    .unwrap(),
            },
        )
        .unwrap();
    }

    let res_generator = |x| factory_resp::PoolInfoResponse {
        id: x,
        address: HumanAddr::from(TEST_POOL),
        dp_address: HumanAddr::from(TEST_POOL_TOKEN),
        dp_total_supply: Uint256::from(TEST_POOL_TOKEN_SUPPLY),
        yield_adapter: HumanAddr::from(TEST_YIELD_ADAPTER),
        yield_token: HumanAddr::from(TEST_YIELD_TOKEN),
        yield_token_balance: Uint256::from(TEST_YIELD_TOKEN_SUPPLY),
        accumulated_reward: Uint256::from(TEST_POOL_REWARD_AMOUNT),
        accumulated_fee: Uint256::from(TEST_POOL_REWARD_FEE),
    };

    let msg = QueryMsg::PoolInfo { pool_id: 0 };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: factory_resp::PoolInfoResponse = from_binary(&bin_res).unwrap();
    assert_eq!(res, res_generator(0));

    let msg = QueryMsg::PoolInfos {
        start_after: Option::None,
        limit: Option::from(10),
    };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: factory_resp::PoolInfosResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::PoolInfosResponse {
            pool_infos: (0..10).map(res_generator).collect(),
        }
    );

    let msg = QueryMsg::PoolInfos {
        start_after: Option::from(10),
        limit: Option::from(10),
    };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: factory_resp::PoolInfosResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::PoolInfosResponse {
            pool_infos: (10..20).map(res_generator).collect(),
        }
    )
}

#[test]
fn query_adapter_info() {
    let mut deps = mock_dependencies(20, &[]);
    let env = utils::initialize(&mut deps);

    let mock_adapters: Vec<HumanAddr> = (0..9)
        .map(|x| HumanAddr::from(format!("yield_adapter_{}", x)))
        .collect();
    for address in mock_adapters {
        let msg = HandleMsg::RegisterAdapter { address };
        let _ = contract::handle(&mut deps, env.clone(), msg).unwrap();
    }

    let res_generator = |x| factory_resp::AdapterInfoResponse {
        address: HumanAddr::from(format!("yield_adapter_{}", x)),
        input_denom: TEST_INPUT_DENOM.to_string(),
        yield_token: HumanAddr::from(TEST_YIELD_TOKEN),
    };

    let msg = QueryMsg::AdapterInfo {
        address: HumanAddr::from(TEST_YIELD_ADAPTER),
    };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: factory_resp::AdapterInfoResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::AdapterInfoResponse {
            address: HumanAddr::from(TEST_YIELD_ADAPTER),
            input_denom: TEST_INPUT_DENOM.to_string(),
            yield_token: HumanAddr::from(TEST_YIELD_TOKEN),
        }
    );

    let msg = QueryMsg::AdapterInfos {
        start_after: Option::from(HumanAddr::from("yield_adapter")),
        limit: Option::from(5),
    };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: factory_resp::AdapterInfosResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::AdapterInfosResponse {
            adapter_infos: (0..5).map(res_generator).collect()
        }
    );

    let msg = QueryMsg::AdapterInfos {
        start_after: Option::from(HumanAddr::from("yield_adapter_5")),
        limit: Option::from(5),
    };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: factory_resp::AdapterInfosResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::AdapterInfosResponse {
            adapter_infos: (6..9).map(res_generator).collect()
        }
    );
}
