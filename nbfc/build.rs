use std::process::Command;

use schemars::schema_for;

#[path = "src/lib.rs"]
mod lib;

use lib::FanControlConfigV2;

fn main() {
    let schema = schema_for!(FanControlConfigV2);
    std::fs::write(
        "ts-types/schema.json",
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap();

    Command::new("ts-types/node_modules/.bin/json2ts")
        .arg("ts-types/schema.json")
        .arg("ts-types/index.d.ts")
        .status()
        .unwrap();
}
