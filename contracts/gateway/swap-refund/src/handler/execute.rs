use cosmwasm_std::{
    Api, BankMsg, CanonicalAddr, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier,
    StdError, StdResult, Storage,
};
use std::ops::Mul;

use crate::state::user_state::UserState;
use crate::state::{config, user, user_state};
use cosmwasm_bignumber::Decimal256;

pub fn check_manager<S: Storage>(storage: &S, env: Env) -> StdResult<()> {
    let config = config::read(storage).unwrap();
    if config.manager.ne(&env.message.sender) {
        return Err(StdError::generic_err(format!(
            "SwapRefund: invalid sender. expected: {}, actual: {}",
            config.manager, env.message.sender,
        )));
    }
    Ok(())
}

pub fn configure<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    manager: Option<HumanAddr>,
    refund_denom: Option<String>,
    base_price: Option<Decimal256>,
) -> StdResult<HandleResponse> {
    check_manager(&deps.storage, env)?;

    let mut config = config::read(&deps.storage).unwrap();

    if let Some(manager) = manager {
        config.manager = manager;
    }
    if let Some(refund_denom) = refund_denom {
        config.refund_denom = refund_denom;
    }
    if let Some(base_price) = base_price {
        config.base_price = base_price;
    }

    config::store(&mut deps.storage, &config).unwrap();

    Ok(HandleResponse::default())
}

pub fn refund<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
) -> StdResult<HandleResponse> {
    check_manager(&deps.storage, env.clone())?;

    let config = config::read(&deps.storage).unwrap();
    if config.manager.ne(&env.message.sender) {
        return Err(StdError::generic_err(format!(
            "SwapRefund: invalid sender. expected: {}, actual: {}",
            config.manager, env.message.sender,
        )));
    }

    let users = user::batch_read(&deps, start_after, limit).unwrap();
    let mut msgs: Vec<CosmosMsg> = Vec::new();
    for (address, info) in users.iter() {
        let user_state =
            user_state::read(&deps.storage, &deps.api.canonical_address(address).unwrap()).unwrap();
        if !user_state.processed {
            if !info.amount.is_zero() {
                let refund_amount = info.amount.mul(config.base_price);

                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: address.clone(),
                    amount: vec![Coin {
                        denom: config.refund_denom.clone(),
                        amount: refund_amount.into(),
                    }],
                }));
            }

            user_state::store(
                &mut deps.storage,
                &deps.api.canonical_address(address).unwrap(),
                &UserState { processed: true },
            )
            .unwrap();
        }
    }

    Ok(HandleResponse {
        messages: msgs,
        log: vec![],
        data: None,
    })
}
