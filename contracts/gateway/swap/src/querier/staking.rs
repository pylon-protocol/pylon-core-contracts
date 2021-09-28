use cosmwasm_std::{
    to_binary, Api, Deps, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, WasmQuery,
};
use pylon_token::gov::{QueryMsg as GovQueryMsg, StakerResponse};

pub fn staker<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    staking_contract: String,
    address: String,
) -> StdResult<StakerResponse> {
    deps.querier
        .query::<StakerResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: staking_contract,
            msg: to_binary(&GovQueryMsg::Staker { address }).unwrap(),
        }))
}
