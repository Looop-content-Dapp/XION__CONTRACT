use cosmwasm_schema::write_api;
use loop_music::msg::{ExecuteMsg, InstantiateMsg, };
use loop_music::schema_types::SchemaQueryMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: SchemaQueryMsg,
    }
}
