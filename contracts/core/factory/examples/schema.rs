use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use core_factory::state::{adapter, config, pool, state};
use pylon_core::factory_msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(adapter::Adapter), &out_dir);
    export_schema(&schema_for!(config::Config), &out_dir);
    export_schema(&schema_for!(pool::Pool), &out_dir);
    export_schema(&schema_for!(state::State), &out_dir);
}
