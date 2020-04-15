/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use dbus::blocking::Connection;

use std::fs::read_dir;

mod interfaces;
use interfaces::ComMusikidFancy;

static CONTROL_CONFIGS_PATH: &str = "/etc/fancy/configs";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "com.musikid.fancy",
        "/com/musikid/fancy",
        std::time::Duration::from_millis(1000),
    );

    let matches = App::new("fancy")
        .setting(AppSettings::SubcommandRequired)
        .version(crate_version!())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("set")
                .about("Set a value")
                .arg(
                    Arg::with_name("target_fans_speeds")
                        .help("Set the fans speeds")
                        .short("f")
                        .long("fans-speeds")
                        .takes_value(true)
                        .value_name("TARGET_FAN_SPEEDS"),
                )
                .arg(
                    Arg::with_name("config")
                        .help("Set the config to use")
                        .short("c")
                        .long("config")
                        .takes_value(true)
                        .value_name("CONFIG"),
                )
                .arg(
                    Arg::with_name("auto")
                        .help("Set auto state")
                        .short("a")
                        .long("auto"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get a value")
                .subcommand(SubCommand::with_name("speeds").about("Get the fans speeds"))
                .subcommand(SubCommand::with_name("temps").about("Get the temperatures"))
                .subcommand(SubCommand::with_name("config").about("Get the current config"))
                .subcommand(SubCommand::with_name("auto").about("Get auto-handle state")),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("Get a list of the available configs")
                .arg(
                    Arg::with_name("recommended")
                        .long("recommended")
                        .help("Filter to get only recommended ones"),
                ),
        )
        .get_matches();

    //TODO: Add the possibility to fetch/update the configs.

    if let Some(matches) = matches.subcommand_matches("get") {
        if let Some(_) = matches.subcommand_matches("speeds") {
            let fans_speeds = proxy.fans_speeds()?;
            for (name, speed) in fans_speeds {
                println!("{}: {:.1}%", name, speed);
            }
        }

        if let Some(_) = matches.subcommand_matches("config") {
            let config = proxy.config()?;
            println!("{}", config);
        }

        if let Some(_) = matches.subcommand_matches("auto") {
            let auto = proxy.auto()?;
            println!("Auto-select thresholds: {}", auto);
        }

        if let Some(_) = matches.subcommand_matches("temps") {
            let temps = proxy.temperatures()?;
            for (sensor, temp) in temps {
                println!("{}: {:.1}°C", sensor, temp);
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("list") {
        let mut configs: Vec<String> = read_dir(CONTROL_CONFIGS_PATH)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter_map(|e| e.file_stem().map(|e| e.to_string_lossy().into_owned()))
            .collect();

        if matches.is_present("recommended") {
            //TODO: Optimize
            let product_name = get_product_name()?;
            configs.retain(|s| {
                bcmp::longest_common_substring(
                    s.as_bytes(),
                    product_name.as_bytes(),
                    bcmp::AlgoSpec::TreeMatch(5),
                )
                .length
                    > 5
            });

            configs.sort_unstable_by(|s1, s2| {
                bcmp::longest_common_substring(
                    s2.as_bytes(),
                    product_name.as_bytes(),
                    bcmp::AlgoSpec::TreeMatch(5),
                )
                .length
                .cmp(
                    &bcmp::longest_common_substring(
                        s1.as_bytes(),
                        product_name.as_bytes(),
                        bcmp::AlgoSpec::TreeMatch(5),
                    )
                    .length,
                )
            });
        } else {
            configs.sort_unstable();
        }

        for conf in configs {
            println!("{}", conf);
        }
    } else if let Some(matches) = matches.subcommand_matches("set") {
        if matches.is_present("target_fans_speeds") {
            let args = matches.values_of("target_fans_speeds").unwrap();
            //TODO: Error handling
            let speeds = args.map(|n| n.parse::<f64>().unwrap()).collect();

            if !matches.is_present("auto") {
                proxy.set_auto(false)?;
            }

            proxy.set_target_fans_speeds(speeds)?;
        }

        if matches.is_present("config") {
            let config = matches.value_of("config").unwrap();
            proxy.set_config(config.to_owned())?;
        }

        if matches.is_present("auto") {
            proxy.set_auto(true)?;
        }
    }

    Ok(())
}

fn get_product_name() -> Result<String, std::io::Error> {
    std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
}
