use cosmwasm_bignumber::Uint256;
use cosmwasm_std::*;
use cw20::Cw20ReceiveMsg;
use pylon_gateway::pool_msg::{Cw20HookMsg, ExecuteMsg};

use crate::error::ContractError;
use crate::state::config;

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => {
            let config = config::read(deps.storage)?;
            if config.share_token.ne(&info.sender.to_string()) {
                return Err(ContractError::Unauthorized {
                    action: "deposit".to_string(),
                    expected: config.share_token,
                    actual: info.sender.to_string(),
                });
            }

            deposit(
                deps,
                env,
                info,
                cw20_msg.sender,
                Uint256::from(cw20_msg.amount),
            )
        }
        _ => Err(ContractError::UnsupportedReceiveMsg {
            typ: stringify!(cw20_msg).to_string(),
        }),
    }
}

pub fn deposit(
    _deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    sender: String,
    amount: Uint256,
) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Update {
                target: Option::Some(sender.clone()),
            })?,
            funds: vec![],
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::DepositInternal { sender, amount })?,
            funds: vec![],
        })))
}

pub fn withdraw(
    _deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint256,
) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Update {
                target: Option::Some(info.sender.to_string()),
            })?,
            funds: vec![],
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::WithdrawInternal {
                sender: info.sender.to_string(),
                amount,
            })?,
            funds: vec![],
        })))
}

pub fn claim(_deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Update {
                target: Option::Some(info.sender.to_string()),
            })?,
            funds: vec![],
        }))
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::ClaimInternal {
                sender: info.sender.to_string(),
            })?,

            funds: vec![],
        })))
}
