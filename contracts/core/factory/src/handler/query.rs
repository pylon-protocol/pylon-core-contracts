use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, QueryRequest, StdResult,
    Storage, WasmQuery,
};
use pylon_core::adapter_msg::QueryMsg as AdapterQueryMsg;
use pylon_core::adapter_resp;
use pylon_core::factory_resp as resp;
use pylon_core::pool_v2_msg::QueryMsg as PoolQueryMSg;
use pylon_core::pool_v2_resp as pool_resp;
use pylon_utils::token;

use crate::state::{adapter, config, pool, state};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = config::read(&deps.storage).unwrap();

    to_binary(&resp::ConfigResponse {
        owner: deps.api.human_address(&config.owner).unwrap(),
        pool_code_id: config.pool_code_id,
        token_code_id: config.token_code_id,
        fee_rate: config.fee_rate,
        fee_collector: deps.api.human_address(&config.fee_collector).unwrap(),
    })
}

fn fetch_pool_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pool_id: u64,
) -> StdResult<resp::PoolInfoResponse> {
    let pool = pool::read(&deps.storage, pool_id).unwrap();
    let pool_addr = deps.api.human_address(&pool.address).unwrap();
    let pool_config: pool_resp::ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_addr.clone(),
            msg: to_binary(&PoolQueryMSg::Config {}).unwrap(),
        }))?;
    let pool_claimable: pool_resp::ClaimableRewardResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_addr.clone(),
            msg: to_binary(&PoolQueryMSg::ClaimableReward {}).unwrap(),
        }))?;

    let dp_total_supply = token::total_supply(
        deps,
        &deps.api.canonical_address(&pool_config.dp_token).unwrap(),
    )?;
    let yield_token_balance = token::balance_of(
        deps,
        &deps
            .api
            .canonical_address(&pool_config.yield_token)
            .unwrap(),
        deps.api.human_address(&pool.address).unwrap(),
    )?;

    Ok(resp::PoolInfoResponse {
        id: pool.id,
        address: pool_addr,
        dp_address: pool_config.dp_token,
        dp_total_supply,
        yield_adapter: pool_config.yield_adapter,
        yield_token: pool_config.yield_token,
        yield_token_balance,
        accumulated_reward: Uint256::from(pool_claimable.amount),
        accumulated_fee: Uint256::from(pool_claimable.fee),
    })
}

pub fn pool_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pool_id: u64,
) -> StdResult<Binary> {
    to_binary(&fetch_pool_info(deps, pool_id)?)
}

pub fn pool_infos<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let pools = pool::batch_read(&deps.storage, start_after, limit).unwrap();

    let mut pool_infos: Vec<resp::PoolInfoResponse> = Vec::new();
    for pool in pools.iter() {
        pool_infos.push(fetch_pool_info(deps, pool.id)?);
    }
    to_binary(&resp::PoolInfosResponse { pool_infos })
}

fn fetch_adapter_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<resp::AdapterInfoResponse> {
    let adapter = adapter::read(&deps.storage, deps.api.canonical_address(&address).unwrap())?;
    let adapter_config: adapter_resp::ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&adapter.address).unwrap(),
            msg: to_binary(&AdapterQueryMsg::Config {}).unwrap(),
        }))?;
    Ok(resp::AdapterInfoResponse {
        address,
        input_denom: adapter_config.input_denom,
        yield_token: adapter_config.yield_token,
    })
}

pub fn adapter_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<Binary> {
    to_binary(&fetch_adapter_info(deps, address)?)
}

pub fn adapter_infos<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let adapters = adapter::batch_read(deps, start_after, limit).unwrap();
    let mut adapter_infos: Vec<resp::AdapterInfoResponse> = Vec::new();
    for adapter in adapters.iter() {
        adapter_infos.push(
            fetch_adapter_info(deps, deps.api.human_address(&adapter.address).unwrap()).unwrap(),
        );
    }
    to_binary(&resp::AdapterInfosResponse { adapter_infos })
}
