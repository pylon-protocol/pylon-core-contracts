#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Addr, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn,
    Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw20::MinterResponse;
use protobuf::Message;
use pylon_core::pool_msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use terraswap::token::InstantiateMsg as Cw20InstantiateMsg;

use crate::error::ContractError;
use crate::handler::core as CoreHandler;
use crate::handler::query as QueryHandler;
use crate::response::MsgInstantiateContractResponse;
use crate::{config, querier};

const INSTANTIATE_REPLY_ID: u64 = 1;

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let mut config = config::Config {
        this: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        beneficiary: deps.api.addr_canonicalize(msg.beneficiary.as_str())?,
        fee_collector: deps.api.addr_canonicalize(msg.fee_collector.as_str())?,
        moneymarket: deps.api.addr_canonicalize(msg.moneymarket.as_str())?,
        stable_denom: String::default(),
        atoken: CanonicalAddr::from(vec![]),
        dp_token: CanonicalAddr::from(vec![]),
    };

    let market_config = querier::anchor::config(deps.as_ref(), &config.moneymarket)?;

    config.stable_denom = market_config.stable_denom.clone();
    config.atoken = deps
        .api
        .addr_canonicalize(market_config.aterra_contract.as_str())?;

    config::store(deps.storage, &config)?;

    Ok(Response::new().add_submessage(SubMsg {
        // Create DP token
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: msg.dp_code_id,
            funds: vec![],
            label: "".to_string(),
            msg: to_binary(&Cw20InstantiateMsg {
                name: format!("Deposit Token - {}", msg.pool_name),
                symbol: "PylonDP".to_string(),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
            })?,
        }
        .into(),
        gas_limit: None,
        id: INSTANTIATE_REPLY_ID,
        reply_on: ReplyOn::Success,
    }))
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
        ExecuteMsg::Configure {
            beneficiary,
            fee_collector,
        } => CoreHandler::configure(deps, env, info, beneficiary, fee_collector),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_REPLY_ID => {
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

            CoreHandler::register_dp_token(deps, env, token_addr)
        }
        _ => Err(ContractError::InvalidReplyId { id: msg.id }),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::DepositAmountOf { owner } => QueryHandler::deposit_amount(deps, env, owner), // dp_token.balanceOf(msg.sender)
        QueryMsg::TotalDepositAmount {} => QueryHandler::total_deposit_amount(deps, env), // dp_token.totalSupply()
        QueryMsg::Config {} => QueryHandler::config(deps, env),                           // config
        QueryMsg::ClaimableReward {} => QueryHandler::claimable_reward(deps, env), // config.strategy.reward()
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
