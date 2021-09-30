use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{
    to_binary, Coin, ContractResult, CosmosMsg, QuerierResult, SystemResult, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::adapter_msg::QueryMsg;
use crate::adapter_resp;
use crate::test_constant::*;

#[derive(Clone)]
pub struct MockAdapter {
    pub target: String,
    pub input_denom: String,
    pub yield_token: String,
    pub exchange_rate: Decimal256,
    pub yield_token_supply: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    DepositStable {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Return stable coins to a user
    /// according to exchange rate
    RedeemStable {},
}

impl Default for MockAdapter {
    fn default() -> Self {
        MockAdapter {
            target: TEST_ADAPTER_TARGET.to_string(),
            input_denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            yield_token: TEST_TOKEN_YIELD.to_string(),
            exchange_rate: Decimal256::from_str(TEST_ADAPTER_EXCHANGE_RATE).unwrap(),
            yield_token_supply: Uint256::from(TEST_TOKEN_YIELD_SUPPLY),
        }
    }
}

impl MockAdapter {
    pub fn handle_query(&self, msg: QueryMsg) -> QuerierResult {
        match msg {
            QueryMsg::Config {} => SystemResult::Ok(ContractResult::Ok(
                to_binary(&adapter_resp::ConfigResponse {
                    input_denom: self.input_denom.clone(),
                    yield_token: self.yield_token.clone(),
                })
                .unwrap(),
            )),
            QueryMsg::ExchangeRate { input_denom: _ } => SystemResult::Ok(ContractResult::Ok(
                to_binary(&adapter_resp::ExchangeRateResponse {
                    exchange_rate: self.exchange_rate,
                    yield_token_supply: self.yield_token_supply,
                })
                .unwrap(),
            )),
            QueryMsg::Deposit { amount } => {
                let msgs: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: self.target.clone(),
                    msg: to_binary(&ExecuteMsg::DepositStable {}).unwrap(),
                    funds: vec![Coin {
                        denom: self.input_denom.clone(),
                        amount: amount.into(),
                    }],
                })];
                SystemResult::Ok(ContractResult::Ok(to_binary(&msgs).unwrap()))
            }
            QueryMsg::Redeem { amount } => {
                let msgs: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: self.target.clone(),
                    msg: to_binary(&Cw20ExecuteMsg::Send {
                        contract: self.yield_token.clone(),
                        amount: amount.into(),
                        msg: to_binary(&Cw20HookMsg::RedeemStable {}).unwrap(),
                    })
                    .unwrap(),
                    funds: vec![],
                })];
                SystemResult::Ok(ContractResult::Ok(to_binary(&msgs).unwrap()))
            }
        }
    }
}
