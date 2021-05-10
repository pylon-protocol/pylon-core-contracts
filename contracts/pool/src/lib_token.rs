use crate::config::{read_config, Config};
use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Extern, HumanAddr, Querier, QueryRequest, StdResult, Storage,
    Uint128, WasmQuery,
};
use cw20::Cw20QueryMsg::{Balance as TokenBalance, TokenInfo};
use cw20::{BalanceResponse, TokenInfoResponse};

pub fn token_balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token: &CanonicalAddr,
    owner: HumanAddr,
) -> StdResult<Uint128> {
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps.api.human_address(token)?,
        msg: to_binary(&TokenBalance { address: owner })?,
    }))?;

    Ok(balance.balance)
}

pub fn token_total_supply<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token: &CanonicalAddr,
) -> StdResult<Uint128> {
    let token_info: TokenInfoResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(token)?,
            msg: to_binary(&TokenInfo {})?,
        }))?;

    Ok(token_info.total_supply)
}
