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
mod wasm {
    use super::contract;
    use cosmwasm_std::{
        do_handle, do_init, do_query, ExternalApi, ExternalQuerier, ExternalStorage,
    };

    #[no_mangle]
    extern "C" fn init(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_init(
            &contract::init::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn handle(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_handle(
            &contract::handle::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn query(msg_ptr: u32) -> u32 {
        do_query(
            &contract::query::<ExternalStorage, ExternalApi, ExternalQuerier>,
            msg_ptr,
        )
    }

    // Other C externs like cosmwasm_vm_version_1, allocate, deallocate are available
    // automatically because we `use cosmwasm_std`.
}
