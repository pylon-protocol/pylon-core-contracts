// core
pub mod contract;
pub mod msg;
pub mod resp;
pub mod state;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
