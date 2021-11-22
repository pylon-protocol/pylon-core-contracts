use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use gateway_pool::state::{config, reward, user};
use pylon_gateway::pool_msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(config::Config), &out_dir);
    export_schema(&schema_for!(config::DepositConfig), &out_dir);
    export_schema(&schema_for!(config::DistributionConfig), &out_dir);
    export_schema(&schema_for!(reward::Reward), &out_dir);
    export_schema(&schema_for!(user::User), &out_dir);
}
