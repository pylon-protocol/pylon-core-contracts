use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use gateway_swap_refund::state::{config, user, user_state};
use pylon_gateway::swap_msg::{HandleMsg, InitMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(HandleMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(config::Config), &out_dir);
    export_schema(&schema_for!(user::User), &out_dir);
    export_schema(&schema_for!(user_state::UserState), &out_dir);
}
