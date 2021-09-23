mod contract;
#[warn(clippy::module_inception)]
pub mod state;

mod handler;
mod querier;

#[cfg(test)]
mod testing;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points_with_migration!(contract);
