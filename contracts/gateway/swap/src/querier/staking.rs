use cosmwasm_std::{
    to_binary, Api, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, Uint128,
    WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerRequest {
    pub address: HumanAddr,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct StakerResponse {
    pub balance: Uint128,
    pub share: Uint128,
}

pub fn staker<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    staking_contract: &HumanAddr,
    address: HumanAddr,
) -> StdResult<StakerResponse> {
    deps.querier
        .query::<StakerResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: staking_contract.clone(),
            msg: to_binary(&StakerRequest { address }).unwrap(),
        }))
}
