use cosmwasm_bignumber::Decimal256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub input_denom: String,
    pub output_token: String,
    pub exchange_rate: Decimal256,
}
