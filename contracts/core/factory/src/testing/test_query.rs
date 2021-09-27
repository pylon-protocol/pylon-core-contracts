use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{from_binary, Uint128};
use pylon_core::factory_msg::{ExecuteMsg, QueryMsg};
use pylon_core::factory_resp;
use pylon_core::test_constant::*;

use crate::contract;
use crate::state::pool;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn query_config() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);

    let msg = QueryMsg::Config {};
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let res: factory_resp::ConfigResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::ConfigResponse {
            owner: TEST_CREATOR.to_string(),
            pool_code_id: TEST_POOL_CODE_ID,
            token_code_id: TEST_TOKEN_CODE_ID,
            fee_rate: Decimal256::percent(TEST_FACTORY_FEE_RATE),
            fee_collector: TEST_FEE_COLLECTOR.to_string(),
        }
    )
}

#[test]
fn query_pool_info() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);
    deps.querier.token.with_balances(&[(
        &TEST_TOKEN_YIELD.to_string(),
        &[(
            &TEST_POOL.to_string(),
            &Uint128::from(TEST_TOKEN_YIELD_SUPPLY),
        )],
    )]);

    for x in 0..20 {
        pool::store(
            &mut deps.storage,
            x,
            &pool::Pool {
                id: x,
                status: pool::Status::Deployed,
                address: TEST_POOL.to_string(),
            },
        )
        .unwrap();
    }

    let res_generator = |x| factory_resp::PoolInfoResponse {
        id: x,
        address: TEST_POOL.to_string(),
        dp_address: TEST_TOKEN_POOL.to_string(),
        dp_total_supply: Uint256::from(TEST_TOKEN_POOL_SUPPLY),
        yield_adapter: TEST_ADAPTER.to_string(),
        yield_token: TEST_TOKEN_YIELD.to_string(),
        yield_token_balance: Uint256::from(TEST_TOKEN_YIELD_SUPPLY),
        accumulated_reward: Uint256::from(TEST_POOL_REWARD_AMOUNT),
        accumulated_fee: Uint256::from(TEST_POOL_REWARD_FEE),
    };

    let msg = QueryMsg::PoolInfo { pool_id: 0 };
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let res: factory_resp::PoolInfoResponse = from_binary(&bin_res).unwrap();
    assert_eq!(res, res_generator(0));

    let msg = QueryMsg::PoolInfos {
        start_after: Option::None,
        limit: Option::from(10),
    };
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
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
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let res: factory_resp::PoolInfosResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::PoolInfosResponse {
            pool_infos: (11..20).map(res_generator).collect(),
        }
    )
}

#[test]
fn query_adapter_info() {
    let mut deps = mock_dependencies(&[]);
    let (env, info) = utils::initialize(&mut deps);

    let mock_adapters: Vec<String> = (0..9).map(|x| format!("{}_{}", TEST_ADAPTER, x)).collect();
    for address in mock_adapters {
        let msg = ExecuteMsg::RegisterAdapter { address };
        let _ = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    }

    let res_generator = |x| factory_resp::AdapterInfoResponse {
        address: format!("{}_{}", TEST_ADAPTER, x),
        input_denom: TEST_INPUT_DENOM.to_string(),
        yield_token: TEST_TOKEN_YIELD.to_string(),
    };

    let msg = QueryMsg::AdapterInfo {
        address: TEST_ADAPTER.to_string(),
    };
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let res: factory_resp::AdapterInfoResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::AdapterInfoResponse {
            address: TEST_ADAPTER.to_string(),
            input_denom: TEST_INPUT_DENOM.to_string(),
            yield_token: TEST_TOKEN_YIELD.to_string(),
        }
    );

    let msg = QueryMsg::AdapterInfos {
        start_after: Option::from(TEST_ADAPTER.to_string()),
        limit: Option::from(5),
    };
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let res: factory_resp::AdapterInfosResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::AdapterInfosResponse {
            adapter_infos: (0..5).map(res_generator).collect()
        }
    );

    let msg = QueryMsg::AdapterInfos {
        start_after: Option::from(format!("{}_5", TEST_ADAPTER)),
        limit: Option::from(5),
    };
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let res: factory_resp::AdapterInfosResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        factory_resp::AdapterInfosResponse {
            adapter_infos: (6..9).map(res_generator).collect()
        }
    );
}
