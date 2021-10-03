use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::*;
use cw20::Cw20ExecuteMsg;
use pylon_utils::tax::deduct_tax;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {
        block_height: Option<u64>,
    },
    EpochState {
        block_height: Option<u64>,
        distributed_interest: Option<Uint256>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner_addr: String,
    pub aterra_contract: String,
    pub interest_model: String,
    pub distribution_model: String,
    pub overseer_contract: String,
    pub collector_contract: String,
    pub distributor_contract: String,
    pub stable_denom: String,
    pub max_borrow_factor: Decimal256,
}

pub fn config(deps: &Deps, market: String) -> StdResult<ConfigResponse> {
    let market_config =
        deps.querier
            .query::<ConfigResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: market,
                msg: to_binary(&QueryMsg::Config {})?,
            }))?;

    Ok(market_config)
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}

pub fn epoch_state(deps: Deps, market: String) -> StdResult<EpochStateResponse> {
    let epoch_state = deps
        .querier
        .query::<EpochStateResponse>(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: market,
            msg: to_binary(&QueryMsg::EpochState {
                block_height: None,
                distributed_interest: None,
            })?,
        }))?;

    Ok(epoch_state)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    DepositStable {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    RedeemStable {},
}

pub fn deposit_stable_msg(
    deps: Deps,
    market: String,
    denom: &str,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: market,
        msg: to_binary(&HandleMsg::DepositStable {})?,
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
