use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use launchpad_lockup::msg;
use launchpad_lockup::state;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(msg::InitMsg), &out_dir);
    export_schema(&schema_for!(msg::HandleMsg), &out_dir);
    export_schema(&schema_for!(msg::QueryMsg), &out_dir);
    export_schema(&schema_for!(msg::Cw20HookMsg), &out_dir);
    export_schema(&schema_for!(state::Config), &out_dir);
    export_schema(&schema_for!(state::Reward), &out_dir);
    export_schema(&schema_for!(state::User), &out_dir);
}
