/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=interfaces/fancy.xml");

    let mut data = String::new();
    std::fs::File::open("interfaces/fancy.xml")
        .unwrap()
        .read_to_string(&mut data)?;
    let interface_code = dbus_codegen::generate(
        &data,
        &dbus_codegen::GenOpts {
            dbuscrate: String::from("dbus_crate"),
            ..Default::default()
        },
    )?;

    let mut file = std::fs::File::create("src/dbus/interfaces.rs")?;
    file.write("#![allow(unused_imports)]\n".as_bytes())?;
    file.write_all(interface_code.as_bytes())?;

    Ok(())
}
