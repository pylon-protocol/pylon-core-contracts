// router
pub mod contract;

// handlers
pub mod handler_exec;
pub mod handler_query;

// state / data structures
pub mod msg;
pub mod resp;
pub mod state;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
