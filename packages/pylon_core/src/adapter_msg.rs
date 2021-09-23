use cosmwasm_bignumber::Uint256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    ExchangeRate { input_denom: String },
    Deposit { amount: Uint256 }, // input_denom
    Redeem { amount: Uint256 },  // yield_token
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {}
