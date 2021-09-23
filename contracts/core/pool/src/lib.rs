// core
pub mod config;
pub mod contract;

// querier
mod handler;
mod querier;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
