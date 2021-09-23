use cosmwasm_std::{Decimal, Uint128};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct MockTax {
    pub rate: Decimal,
    // this lets us iterate over all pairs that match the first string
    pub caps: HashMap<String, Uint128>,
}

impl MockTax {
    #[allow(dead_code)]
    pub fn new(rate: Decimal, caps: &[(&String, &Uint128)]) -> Self {
        let mut owner_map: HashMap<String, Uint128> = HashMap::new();
        for (denom, cap) in caps.iter() {
            owner_map.insert(denom.to_string(), **cap);
        }

        MockTax {
            rate,
            caps: owner_map,
        }
    }
}
