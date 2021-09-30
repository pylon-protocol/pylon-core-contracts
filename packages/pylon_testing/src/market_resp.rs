use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct EpochStateResponse {
    pub exchange_rate: Decimal256,
    pub aterra_supply: Uint256,
}
