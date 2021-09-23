mod anchor;
pub mod config;
mod contract;
pub mod msg;

#[cfg(test)]
mod testing;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
