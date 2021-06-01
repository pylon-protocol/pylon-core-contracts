use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage, WasmQuery,
};
use cw20::BalanceResponse;

pub fn balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token: &CanonicalAddr,
    owner: &HumanAddr,
) -> StdResult<Uint256> {
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: deps.api.human_address(token)?,
        key: Default::default(),
    }))?;
}
