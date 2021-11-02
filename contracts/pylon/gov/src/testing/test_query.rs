use cosmwasm_std::from_binary;
use cosmwasm_std::testing::mock_env;
use pylon_token::gov_msg::QueryMsg;
use pylon_token::gov_resp::APIVersionResponse;

use crate::constant::API_VERSION;
use crate::contract;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils::mock_instantiate;

#[test]
fn query_api_version() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = QueryMsg::APIVersion {};
    let raw_resp = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let resp: APIVersionResponse = from_binary(&raw_resp).unwrap();
    assert_eq!(resp.version, API_VERSION.to_string());
}
