use cosmwasm_std::*;
use cw20::Cw20HandleMsg;
use pylon_testing::market_msg::{Cw20HookMsg, ExecuteMsg, QueryMsg};
use pylon_testing::market_resp::{ConfigResponse, EpochStateResponse};
use pylon_utils::tax::deduct_tax;

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: String,
) -> StdResult<ConfigResponse> {
    deps.querier
        .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: HumanAddr::from(market),
            msg: to_binary(&QueryMsg::Config {})?,
        }))
}

pub fn epoch_state<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: String,
) -> StdResult<EpochStateResponse> {
    deps.querier
        .query::<EpochStateResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: HumanAddr::from(market),
            msg: to_binary(&QueryMsg::EpochState {
                block_height: None,
                distributed_interest: None,
            })?,
        }))
}

pub fn deposit_stable_msg<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: String,
    denom: &str,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: HumanAddr::from(market),
        msg: to_binary(&ExecuteMsg::DepositStable {})?,
        send: vec![deduct_tax(
            deps,
            Coin {
                denom: denom.to_string(),
                amount,
            },
        )?],
    })])
}

pub fn redeem_stable_msg<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    market: String,
    token: String,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: HumanAddr::from(token),
        msg: to_binary(&Cw20HandleMsg::Send {
            contract: HumanAddr::from(market),
            amount,
            msg: Option::from(to_binary(&Cw20HookMsg::RedeemStable {})?),
        })?,
        send: vec![],
    })])
}
