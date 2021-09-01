use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    to_binary, Api, Binary, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, WasmQuery,
};
use pylon_core::adapter_msg::QueryMsg as AdapterQueryMsg;
use pylon_core::adapter_resp;
use pylon_core::factory_resp as resp;
use pylon_core::pool_v2_msg::QueryMsg as PoolQueryMSg;
use pylon_core::pool_v2_resp as pool_resp;

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

pub fn pool_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pool_id: u64,
) -> StdResult<Binary> {
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

    to_binary(&resp::PoolInfoResponse {
        id: pool.id,
        address: pool_addr,
        dp_address: pool_config.dp_token,
        yield_adapter: pool_config.yield_adapter,
        accumulated_fee: Uint256::from(pool_claimable.fee),
    })
}

pub fn adapter_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
) -> StdResult<Binary> {
    let adapter = adapter::read(&deps.storage, deps.api.canonical_address(&address).unwrap())?;
    let adapter_config: adapter_resp::ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(&adapter.address).unwrap(),
            msg: to_binary(&AdapterQueryMsg::Config {}).unwrap(),
        }))?;

    to_binary(&resp::AdapterInfoResponse {
        address,
        input_denom: adapter_config.input_denom,
        yield_token: adapter_config.yield_token,
    })
}
