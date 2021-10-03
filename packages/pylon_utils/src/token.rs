use cosmwasm_std::*;

use cosmwasm_bignumber::Uint256;
use cw20::Cw20QueryMsg;
use cw20::{BalanceResponse, TokenInfoResponse};

pub fn balance_of(deps: Deps, token: String, owner: String) -> StdResult<Uint256> {
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: token,
        msg: to_binary(&Cw20QueryMsg::Balance { address: owner })?,
    }))?;

    Ok(Uint256::from(balance.balance))
}

pub fn total_supply(deps: Deps, token: String) -> StdResult<Uint256> {
    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: token,
            msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
        }))?;

    Ok(Uint256::from(token_info.total_supply))
}
