use cosmwasm_std::{
    to_binary, Api, Binary, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, WasmQuery,
};

use pylon_core::factory_resp as resp;

use crate::state::{adapter, config, pool, state};
use cosmwasm_bignumber::Uint256;
use pylon_core::pool_v2_msg::QueryMsg;
use pylon_core::pool_v2_resp::{ClaimableRewardResponse, ConfigResponse};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = config::read(&deps.storage)?;

    to_binary(&resp::ConfigResponse {
        owner: deps.api.human_address(&config.owner)?,
        pool_code_id: config.pool_code_id,
        token_code_id: config.token_code_id,
        fee_rate: config.fee_rate,
        fee_collector: deps.api.human_address(&config.fee_collector)?,
    })
}

pub fn state<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let state = state::read(&deps.storage)?;

    to_binary(&resp::StateResponse {
        next_pool_id: state.next_pool_id,
    })
}

pub fn pool_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pool_id: u64,
) -> StdResult<Binary> {
    let pool = pool::read(&deps.storage, pool_id)?;
    let pool_addr = deps.api.human_address(&pool.address)?;
    let pool_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_addr.clone(),
            msg: to_binary(&QueryMsg::Config {})?,
        }))?;
    let pool_claimable: ClaimableRewardResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_addr.clone(),
            msg: to_binary(&QueryMsg::ClaimableReward {})?,
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
    let _adapter = adapter::read(&deps.storage, deps.api.canonical_address(&address)?)?;

    to_binary(&resp::AdapterInfoResponse { address })
}
