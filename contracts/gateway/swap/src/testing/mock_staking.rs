use cosmwasm_std::{to_binary, HumanAddr, QuerierResult, Uint128};
use std::collections::HashMap;

use crate::querier::staking::{StakerRequest, StakerResponse};

#[derive(Clone, Default)]
pub struct MockStaking {
    pub infos: HashMap<HumanAddr, StakerResponse>,
}

impl MockStaking {
    pub fn handle_query(&self, msg: StakerRequest) -> QuerierResult {
        let def = &StakerResponse {
            balance: Uint128::zero(),
            share: Uint128::zero(),
        };
        Ok(to_binary(match self.infos.get(&msg.address) {
            Some(info) => info,
            None => def,
        }))
    }
}

impl MockStaking {
    pub fn new(infos: &[(&String, StakerResponse)]) -> Self {
        MockStaking {
            infos: infos_to_map(infos),
        }
    }
}

pub fn infos_to_map(infos: &[(&String, StakerResponse)]) -> HashMap<HumanAddr, StakerResponse> {
    let mut info_map: HashMap<HumanAddr, StakerResponse> = HashMap::new();
    for (staker, staker_info) in infos.iter() {
        info_map.insert(HumanAddr::from(staker.to_string()), staker_info.clone());
    }
    info_map
}
