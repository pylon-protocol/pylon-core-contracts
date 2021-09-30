pub mod config;
pub mod msg;

mod contract;
mod market;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
