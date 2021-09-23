use cosmwasm_std::{
    to_binary, Api, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, WasmQuery,
};
use pylon_token::gov::{QueryMsg as GovQueryMsg, StakerResponse};

pub fn staker<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    staking_contract: &HumanAddr,
    address: HumanAddr,
) -> StdResult<StakerResponse> {
    deps.querier
        .query::<StakerResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: staking_contract.clone(),
            msg: to_binary(&GovQueryMsg::Staker { address }).unwrap(),
        }))
}
