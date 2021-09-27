use cosmwasm_std::*;
use pylon_core::adapter_msg::QueryMsg as AdapterQueryMsg;
use pylon_core::adapter_resp;
use pylon_core::factory_resp as resp;
use pylon_core::pool_v2_msg::QueryMsg as PoolQueryMSg;
use pylon_core::pool_v2_resp as pool_resp;
use pylon_utils::token;

use crate::state::{adapter, config, pool};

pub fn config(deps: Deps) -> StdResult<Binary> {
    let config = config::read(deps.storage).unwrap();

    to_binary(&resp::ConfigResponse {
        owner: config.owner,
        pool_code_id: config.pool_code_id,
        token_code_id: config.token_code_id,
        fee_rate: config.fee_rate,
        fee_collector: config.fee_collector,
    })
}

fn fetch_pool_info(deps: Deps, pool_id: u64) -> StdResult<resp::PoolInfoResponse> {
    let pool = pool::read(deps.storage, pool_id).unwrap();
    let pool_config: pool_resp::ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool.address.clone(),
            msg: to_binary(&PoolQueryMSg::Config {}).unwrap(),
        }))?;
    let pool_claimable: pool_resp::ClaimableRewardResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool.address.clone(),
            msg: to_binary(&PoolQueryMSg::ClaimableReward {}).unwrap(),
        }))?;

    let dp_total_supply = token::total_supply(deps, pool_config.dp_token.clone())?;
    let yield_token_balance =
        token::balance_of(deps, pool_config.yield_token.clone(), pool.address.clone())?;

    Ok(resp::PoolInfoResponse {
        id: pool.id,
        address: pool.address,
        dp_address: pool_config.dp_token,
        dp_total_supply,
        yield_adapter: pool_config.yield_adapter,
        yield_token: pool_config.yield_token,
        yield_token_balance,
        accumulated_reward: pool_claimable.amount,
        accumulated_fee: pool_claimable.fee,
    })
}

pub fn pool_info(deps: Deps, pool_id: u64) -> StdResult<Binary> {
    to_binary(&fetch_pool_info(deps, pool_id)?)
}

pub fn pool_infos(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Binary> {
    let pools = pool::batch_read(deps.storage, start_after, limit).unwrap();

    let mut pool_infos: Vec<resp::PoolInfoResponse> = Vec::new();
    for pool in pools.iter() {
        pool_infos.push(fetch_pool_info(deps, pool.id)?);
    }
    to_binary(&resp::PoolInfosResponse { pool_infos })
}

fn fetch_adapter_info(deps: Deps, address: String) -> StdResult<resp::AdapterInfoResponse> {
    let adapter = adapter::read(deps.storage, address.clone())?;
    let adapter_config: adapter_resp::ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: adapter.address,
            msg: to_binary(&AdapterQueryMsg::Config {}).unwrap(),
        }))?;
    Ok(resp::AdapterInfoResponse {
        address,
        input_denom: adapter_config.input_denom,
        yield_token: adapter_config.yield_token,
    })
}

pub fn adapter_info(deps: Deps, address: String) -> StdResult<Binary> {
    to_binary(&fetch_adapter_info(deps, address)?)
}

pub fn adapter_infos(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let adapters = adapter::batch_read(deps.storage, start_after, limit).unwrap();
    let mut adapter_infos: Vec<resp::AdapterInfoResponse> = Vec::new();
    for adapter in adapters.iter() {
        adapter_infos.push(fetch_adapter_info(deps, adapter.address.clone())?);
    }

    to_binary(&resp::AdapterInfosResponse { adapter_infos })
}
