use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{to_binary, Coin, CosmosMsg, HumanAddr, QuerierResult, WasmMsg};
use cw20::Cw20HandleMsg;
use pylon_core::adapter_msg::QueryMsg;
use pylon_core::adapter_resp;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::testing::constants::{
    TEST_ADAPTER_EXCHANGE_RATE, TEST_ADAPTER_INPUT_DENOM, TEST_ADAPTER_TARGET, TEST_TOKEN_YIELD,
    TEST_TOKEN_YIELD_SUPPLY,
};

#[derive(Clone)]
pub struct MockAdapter {
    pub target: HumanAddr,
    pub input_denom: String,
    pub yield_token: HumanAddr,
    pub exchange_rate: Decimal256,
    pub yield_token_supply: Uint256,
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

impl Default for MockAdapter {
    fn default() -> Self {
        MockAdapter {
            target: HumanAddr::from(TEST_ADAPTER_TARGET),
            input_denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            yield_token: HumanAddr::from(TEST_TOKEN_YIELD),
            exchange_rate: Decimal256::from_str(TEST_ADAPTER_EXCHANGE_RATE).unwrap(),
            yield_token_supply: Uint256::from(TEST_TOKEN_YIELD_SUPPLY),
        }
    }
}

impl MockAdapter {
    pub fn handle_query(&self, msg: QueryMsg) -> QuerierResult {
        match msg {
            QueryMsg::Config {} => Ok(to_binary(&adapter_resp::ConfigResponse {
                input_denom: self.input_denom.clone(),
                yield_token: self.yield_token.clone(),
            })),
            QueryMsg::ExchangeRate { input_denom: _ } => {
                Ok(to_binary(&adapter_resp::ExchangeRateResponse {
                    exchange_rate: self.exchange_rate.clone(),
                    yield_token_supply: self.yield_token_supply.clone(),
                }))
            }
            QueryMsg::Deposit { amount } => {
                let msgs: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: self.target.clone(),
                    msg: to_binary(&HandleMsg::DepositStable {}).unwrap(),
                    send: vec![Coin {
                        denom: self.input_denom.clone(),
                        amount: amount.into(),
                    }],
                })];
                Ok(to_binary(&msgs))
            }
            QueryMsg::Redeem { amount } => {
                let msgs: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: self.target.clone(),
                    msg: to_binary(&Cw20HandleMsg::Send {
                        contract: self.yield_token.clone(),
                        amount: amount.into(),
                        msg: Option::from(to_binary(&Cw20HookMsg::RedeemStable {}).unwrap()),
                    })
                    .unwrap(),
                    send: vec![],
                })];
                Ok(to_binary(&msgs))
            }
        }
    }
}
