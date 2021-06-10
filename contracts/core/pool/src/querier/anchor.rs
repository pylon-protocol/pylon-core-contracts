use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Api, CanonicalAddr, Coin, CosmosMsg, Extern, HumanAddr, Querier, QueryRequest,
    StdResult, Storage, Uint128, WasmMsg, WasmQuery,
};
use cw20::Cw20HandleMsg;
use moneymarket::querier::deduct_tax;

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
    pub owner_addr: HumanAddr,
    pub aterra_contract: HumanAddr,
    pub interest_model: HumanAddr,
    pub distribution_model: HumanAddr,
    pub overseer_contract: HumanAddr,
    pub collector_contract: HumanAddr,
    pub distributor_contract: HumanAddr,
    pub stable_denom: String,
    pub max_borrow_factor: Decimal256,
}

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
) -> StdResult<ConfigResponse> {
    let market_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(market)?,
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

pub fn epoch_state<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
) -> StdResult<EpochStateResponse> {
    let epoch_state: EpochStateResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: deps.api.human_address(market)?,
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

pub fn deposit_stable_msg<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
    denom: &str,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(market)?,
        msg: to_binary(&HandleMsg::DepositStable {})?,
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
    deps: &Extern<S, A, Q>,
    market: &CanonicalAddr,
    token: &CanonicalAddr,
    amount: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(token)?,
        msg: to_binary(&Cw20HandleMsg::Send {
            contract: deps.api.human_address(market)?,
            amount,
            msg: Option::from(to_binary(&Cw20HookMsg::RedeemStable {})?),
        })?,
        send: vec![],
    })])
}
