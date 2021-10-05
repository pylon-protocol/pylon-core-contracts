use crate::contract;
use crate::mock_querier::mock_dependencies;
use crate::querier::anchor::{
    ConfigResponse, EpochStateResponse, QueryMsg as AnchorQueryMsg, QueryMsg,
};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary};
use pylon_core::pool_msg::InstantiateMsg;

const MONEY_MARKET: &str = "money-market";

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    let info = mock_info("owner", &[]);

    deps.querier.register_wasm_smart_query_handler(
        MONEY_MARKET.to_string(),
        Box::new(|x| match from_binary::<AnchorQueryMsg>(x).unwrap() {
            QueryMsg::Config {} => to_binary(&ConfigResponse {
                owner_addr: "".to_string(),
                aterra_contract: "token-aust".to_string(),
                interest_model: "".to_string(),
                distribution_model: "".to_string(),
                overseer_contract: "".to_string(),
                collector_contract: "".to_string(),
                distributor_contract: "".to_string(),
                stable_denom: "uusd".to_string(),
                max_borrow_factor: Default::default(),
            }),
            QueryMsg::EpochState { .. } => to_binary(&EpochStateResponse {
                exchange_rate: Default::default(),
                aterra_supply: Default::default(),
            }),
        }),
    );

    let msg = InstantiateMsg {
        pool_name: "test-pool".to_string(),
        beneficiary: "test-beneficiary".to_string(),
        fee_collector: "test-fee-collector".to_string(),
        moneymarket: MONEY_MARKET.to_string(),
        dp_code_id: 666,
    };
    let resp = contract::instantiate(deps.as_mut(), env, info, msg)
        .expect("testing: should init contract");
    println!("{:?}", resp);
}
