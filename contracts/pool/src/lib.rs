// router
pub mod contract;

// handlers
pub mod handler_exec;
pub mod handler_query;

// querier
pub mod lib_anchor;
pub mod lib_pool;
pub mod lib_token;

// config / data structures
pub mod config;
pub mod msg;
pub mod resp;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
