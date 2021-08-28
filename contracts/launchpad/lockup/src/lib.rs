// core
pub mod contract;
pub mod state;

mod handler;
mod lib_staking;
mod migrate;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
