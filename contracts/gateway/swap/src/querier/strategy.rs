use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Deps, StdResult};
use pylon_gateway::swap_msg::Strategy;

use crate::state::{config, user};

pub fn claimable_token_of(deps: Deps, time: u64, address: String) -> StdResult<Uint256> {
    let sender = &deps.api.addr_canonicalize(address.as_str()).unwrap();
    let config = config::read(deps.storage).load().unwrap();
    let user = user::read(deps.storage, sender).unwrap();

    let mut count = 0;
    let mut ratio = Decimal256::zero();
    for strategy in config.distribution_strategy.iter() {
        ratio += match strategy {
            Strategy::Lockup {
                release_time,
                release_amount,
            } => {
                if time < *release_time {
                    Decimal256::zero()
                } else {
                    count += 1;
                    *release_amount
                }
            }
            Strategy::Vesting {
                release_start_time,
                release_finish_time,
                release_amount,
            } => {
                if &time <= release_start_time {
                    Decimal256::zero()
                } else if release_finish_time < &time {
                    count += 1;
                    *release_amount
                } else {
                    *release_amount
                        * Decimal256::from_ratio(
                            time - release_start_time,
                            release_finish_time - release_start_time,
                        )
                }
            }
        };
    }
    if config.distribution_strategy.len() == count {
        ratio = Decimal256::one();
    }

    let unlocked = user.swapped_out * ratio;
    let claimable_token = unlocked - user.swapped_out_claimed;

    Ok(claimable_token)
}
