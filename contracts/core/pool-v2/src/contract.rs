#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cw20::MinterResponse;
use pylon_core::factory_msg::ExecuteMsg as FactoryExecuteMsg;
use pylon_core::pool_v2_msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use terraswap::token::InstantiateMsg as Cw20InitMsg;

use crate::error::ContractError;
use crate::handler::core as CoreHandler;
use crate::handler::core::register_dp_token;
use crate::handler::query as QueryHandler;
use crate::querier::adapter;
use crate::response::MsgInstantiateContractResponse;
use crate::state::config;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, SubMsg, WasmMsg,
};
use protobuf::Message;

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let adapter_config = adapter::config(deps.as_ref(), msg.yield_adapter.to_string())?;
    let config = config::Config {
        id: msg.pool_id,
        name: msg.pool_name.clone(),
        factory: info.sender.to_string(),
        beneficiary: msg.beneficiary,
        yield_adapter: msg.yield_adapter.to_string(),
        input_denom: adapter_config.input_denom,
        yield_token: adapter_config.yield_token,
        dp_token: "".to_string(),
    };

    config::store(deps.storage, &config)?;

    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: None,
                code_id: msg.dp_code_id,
                msg: to_binary(&Cw20InitMsg {
                    name: "Pylon Deposit Pool Token".to_string(),
                    symbol: "DPvTwo".to_string(), // naming sucks
                    decimals: 6u8,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                })?,
                funds: vec![],
                label: "".to_string(),
            }),
            1,
        ))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.factory,
            msg: to_binary(&FactoryExecuteMsg::RegisterPool {
                pool_id: msg.pool_id,
            })?,
            funds: vec![],
        })))
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => CoreHandler::receive(deps, env, info, msg),
        ExecuteMsg::Deposit {} => CoreHandler::deposit(deps, env, info),
        ExecuteMsg::Earn {} => CoreHandler::earn(deps, env, info),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => {
            // get new token's contract address
            let res: MsgInstantiateContractResponse = Message::parse_from_bytes(
                msg.result.unwrap().data.unwrap().as_slice(),
            )
            .map_err(|_| {
                ContractError::Std(StdError::parse_err(
                    "MsgInstantiateContractResponse",
                    "failed to parse data",
                ))
            })?;
            let token_addr = Addr::unchecked(res.get_contract_address());

            register_dp_token(deps, env, token_addr)
        }
        _ => Err(ContractError::InvalidReplyId { id: msg.id }),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps, env),
        QueryMsg::DepositAmountOf { owner } => QueryHandler::deposit_amount(deps, env, owner),
        QueryMsg::TotalDepositAmount {} => QueryHandler::total_deposit_amount(deps, env),
        QueryMsg::ClaimableReward {} => QueryHandler::claimable_reward(deps, env),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
