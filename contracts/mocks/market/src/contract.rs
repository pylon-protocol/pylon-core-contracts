#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    coin, from_binary, to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Reply, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, MinterResponse};
use protobuf::Message;
use pylon_testing::market_msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use pylon_testing::market_resp::{ConfigResponse, EpochStateResponse};
use std::ops::{Div, Mul};
use terraswap::token::InstantiateMsg as Cw20InstantiateMsg;

use crate::error::ContractError;
use crate::response::MsgInstantiateContractResponse;
use crate::state;
use cosmwasm_bignumber::Uint256;

const REPLY_INIT_OUTPUT_TOKEN: u64 = 1;

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    state::config_w(deps.storage)
        .save(&state::Config {
            owner: info.sender.to_string(),
            input_denom: msg.input_denom,
            output_token: "".to_string(),
            exchange_rate: msg.exchange_rate,
        })
        .unwrap();
    Ok(Response::new().add_submessage(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id: msg.token_code_id,
            msg: to_binary(&Cw20InstantiateMsg {
                name: "MarketOutputToken".to_string(),
                symbol: "MOT".to_string(), // naming sucks
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
        REPLY_INIT_OUTPUT_TOKEN,
    )))
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => {
            let config = state::config_r(deps.storage).load().unwrap();
            if config.output_token != info.sender {
                return Err(ContractError::Unauthorized {
                    action: "cw20_receive".to_string(),
                    expected: config.output_token,
                    actual: info.sender.to_string(),
                });
            }
            match from_binary(&cw20_msg.msg) {
                Ok(Cw20HookMsg::RedeemStable {}) => {
                    let return_amount = Uint256::from(cw20_msg.amount).mul(config.exchange_rate);
                    Ok(Response::new()
                        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                            contract_addr: config.output_token,
                            msg: to_binary(&Cw20ExecuteMsg::Burn {
                                amount: cw20_msg.amount,
                            })
                            .unwrap(),
                            funds: vec![],
                        }))
                        .add_message(CosmosMsg::Bank(BankMsg::Send {
                            to_address: cw20_msg.sender,
                            amount: vec![coin(return_amount.into(), config.input_denom)],
                        })))
                }
                _ => Err(ContractError::UnsupportedReceiveMsg {
                    typ: stringify!(unmarshalled).to_string(),
                }),
            }
        }
        ExecuteMsg::DepositStable {} => {
            let config = state::config_r(deps.storage).load().unwrap();
            let received = info
                .funds
                .iter()
                .find(|c| c.denom == config.input_denom)
                .map(|c| Uint256::from(c.amount))
                .unwrap_or_else(Uint256::zero);
            if received.is_zero() {
                return Err(ContractError::NotAllowZeroAmount {});
            }
            let return_amount = received.div(config.exchange_rate);

            Ok(
                Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: config.output_token,
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: info.sender.to_string(),
                        amount: return_amount.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                })),
            )
        }
        ExecuteMsg::Configure { exchange_rate } => {
            state::config_w(deps.storage).update(|mut config| {
                if config.owner != info.sender {
                    return Err(ContractError::Unauthorized {
                        action: "configure".to_string(),
                        expected: config.owner,
                        actual: info.sender.to_string(),
                    });
                }
                config.exchange_rate = exchange_rate;
                Ok(config)
            })?;
            Ok(Response::default())
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        REPLY_INIT_OUTPUT_TOKEN => {
            // output token
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

            state::config_w(deps.storage).update(|mut config| {
                if !config.output_token.is_empty() {
                    return Err(ContractError::Unauthorized {
                        action: "reply_init_output_token".to_string(),
                        expected: "<empty>".to_string(),
                        actual: config.output_token,
                    });
                }
                config.output_token = token_addr.to_string();
                Ok(config)
            })?;
            Ok(Response::default())
        }
        _ => Err(ContractError::InvalidReplyId { id: msg.id }),
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = state::config_r(deps.storage).load().unwrap();
            to_binary(&ConfigResponse {
                owner_addr: config.owner,
                aterra_contract: config.output_token,
                interest_model: "".to_string(),
                distribution_model: "".to_string(),
                overseer_contract: "".to_string(),
                collector_contract: "".to_string(),
                distributor_contract: "".to_string(),
                stable_denom: config.input_denom,
                max_borrow_factor: Default::default(),
            })
        }
        QueryMsg::EpochState { .. } => {
            let config = state::config_r(deps.storage).load().unwrap();
            to_binary(&EpochStateResponse {
                exchange_rate: config.exchange_rate,
                aterra_supply: Default::default(),
            })
        }
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
