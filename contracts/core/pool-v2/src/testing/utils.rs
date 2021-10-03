use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{ContractResult, Env, OwnedDeps, Reply, SubMsgExecutionResponse};
use pylon_core::pool_v2_msg::InstantiateMsg;
use pylon_core::test_constant::*;

use crate::contract;
use crate::response::MsgInstantiateContractResponse;
use crate::testing::mock_querier::CustomMockQuerier;
use protobuf::Message;

pub fn init_msg() -> InstantiateMsg {
    InstantiateMsg {
        pool_id: TEST_POOL_ID,
        pool_name: TEST_POOL_NAME.to_string(),
        beneficiary: TEST_BENEFICIARY.to_string(),
        yield_adapter: TEST_ADAPTER.to_string(),
        dp_code_id: TEST_TOKEN_CODE_ID,
    }
}

pub fn initialize(deps: &mut OwnedDeps<MockStorage, MockApi, CustomMockQuerier>) -> Env {
    let env = mock_env();
    let info = mock_info(TEST_FACTORY, &[]);
    let msg = init_msg();
    let _res = contract::instantiate(deps.as_mut(), env.clone(), info, msg)
        .expect("testing: contract initialized");

    let mut token_inst_res = MsgInstantiateContractResponse::new();
    token_inst_res.set_contract_address(TEST_TOKEN_POOL.to_string());
    let reply_msg = Reply {
        id: 1,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: Some(token_inst_res.write_to_bytes().unwrap().into()),
        }),
    };
    let _res = contract::reply(deps.as_mut(), env.clone(), reply_msg)
        .expect("testing: dp token address registered");

    env
}
