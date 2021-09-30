use cosmwasm_std::*;
use cw20::Cw20ExecuteMsg;
use pylon_testing::market_msg::{Cw20HookMsg, ExecuteMsg, QueryMsg};
use pylon_testing::market_resp::{ConfigResponse, EpochStateResponse};
use pylon_utils::tax::deduct_tax;

pub fn config(deps: Deps, market: String) -> StdResult<ConfigResponse> {
    deps.querier
        .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: market,
            msg: to_binary(&QueryMsg::Config {})?,
        }))
}

pub fn epoch_state(deps: Deps, market: String) -> StdResult<EpochStateResponse> {
    deps.querier
        .query::<EpochStateResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: market,
            msg: to_binary(&QueryMsg::EpochState {
                block_height: None,
                distributed_interest: None,
            })?,
        }))
}

pub fn deposit_stable_msg(
    deps: Deps,
    market: String,
    denom: &str,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: market,
        msg: to_binary(&ExecuteMsg::DepositStable {})?,
        funds: vec![deduct_tax(
            deps,
            Coin {
                denom: denom.to_string(),
                amount,
            },
        )?],
    })])
}

pub fn redeem_stable_msg(
    _deps: Deps,
    market: String,
    token: String,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token,
        msg: to_binary(&Cw20ExecuteMsg::Send {
            contract: market,
            amount,
            msg: to_binary(&Cw20HookMsg::RedeemStable {})?,
        })?,
        funds: vec![],
    })])
}
