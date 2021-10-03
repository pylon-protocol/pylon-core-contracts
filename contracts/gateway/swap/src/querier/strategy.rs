use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, StdResult};
use pylon_gateway::cap_strategy_msg::QueryMsg;
use pylon_gateway::cap_strategy_resp as resp;

use crate::state::{config, user};
use pylon_gateway::swap_msg::Strategy;

pub fn available_cap_of(deps: Deps, strategy: String, address: String) -> StdResult<Uint256> {
    let user = user::read(
        deps.storage,
        &deps.api.addr_canonicalize(address.as_str()).unwrap(),
    )
    .unwrap();
    let resp: resp::AvailableCapOfResponse = deps.querier.query_wasm_smart(
        strategy,
        &QueryMsg::AvailableCapOf {
            amount: user.swapped_in,
        },
    )?;
    Ok(resp.amount)
}

pub fn claimable_token_of(deps: Deps, time: u64, address: String) -> StdResult<Uint256> {
    let sender = &deps.api.addr_canonicalize(address.as_str()).unwrap();
    let config = config::read(deps.storage).load().unwrap();
    let user = user::read(deps.storage, sender).unwrap();

    let mut ratio = Decimal256::zero();
    for strategy in config.distribution_strategy.iter() {
        ratio += match strategy {
            Strategy::Lockup {
                release_time,
                release_amount,
            } => {
                if release_time < &time {
                    Decimal256::zero()
                } else {
                    *release_amount
                }
            }
            Strategy::Vesting {
                release_start_time,
                release_finish_time,
                release_amount,
            } => {
                if &time < release_start_time {
                    Decimal256::zero()
                } else if release_finish_time < &time {
                    *release_amount
                } else {
                    *release_amount
                        / Decimal256::from_ratio(
                            time - release_start_time,
                            release_finish_time - release_start_time,
                        )
                }
            }
        };
    }

    let accumulated = user.swapped_out * ratio;
    let claimable_token = accumulated - user.swapped_out_claimed;

    Ok(claimable_token)
}
