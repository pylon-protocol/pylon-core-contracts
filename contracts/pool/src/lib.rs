// core
pub mod config;
pub mod contract;
pub mod msg;
pub mod resp;

// handlers
mod handler_exec;
mod handler_query;

// querier
mod lib_anchor;
mod lib_er_feeder;
mod lib_pool;
mod lib_token;

// config / data structures

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
