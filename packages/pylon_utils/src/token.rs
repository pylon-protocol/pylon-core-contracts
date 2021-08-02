use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage,
    WasmQuery,
};

use cosmwasm_bignumber::Uint256;
use cw20::Cw20QueryMsg;
use cw20::{BalanceResponse, TokenInfoResponse};

pub fn balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token: &CanonicalAddr,
    owner: HumanAddr,
) -> StdResult<Uint256> {
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps.api.human_address(token)?,
        msg: to_binary(&Cw20QueryMsg::Balance { address: owner })?,
    }))?;

    Ok(Uint256::from(balance.balance))
}

pub fn total_supply<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token: &CanonicalAddr,
) -> StdResult<Uint256> {
    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(token)?,
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))?;

    Ok(Uint256::from(token_info.total_supply))
}
