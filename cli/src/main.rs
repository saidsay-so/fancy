/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use clap::values_t;
use dbus::blocking::Connection;

use std::fs::read_dir;

mod app;
mod interfaces;
use app::get_app;
use interfaces::ComMusikidFancy;

static CONTROL_CONFIGS_PATH: &str = "/etc/fancy/configs";

fn main() -> Result<(), anyhow::Error> {
    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "com.musikid.fancy",
        "/com/musikid/fancy",
        std::time::Duration::from_millis(1000),
    );

    let matches = get_app().get_matches();

    if let Some(matches) = matches.subcommand_matches("get") {
        if matches.is_present("speeds") || matches.is_present("status") {
            if matches.is_present("status") {
                println!("Fans speeds");
            }
            let fans_speeds = proxy.fans_speeds()?;
            let names = proxy.fans_names()?;
            for (name, speed) in names.iter().zip(fans_speeds) {
                println!("{}: {:.1}%", name, speed);
            }
        }
        if matches.is_present("config") || matches.is_present("status") {
            if matches.is_present("status") {
                print!("\nConfig: ");
            }
            let config = proxy.config()?;
            println!("{}", config);
        }
        if matches.is_present("auto") || matches.is_present("status") {
            if matches.is_present("status") {
                print!("\nAuto-select thresholds: ");
            }
            let auto = proxy.auto()?;
            println!("{}", auto);
        }
        if matches.is_present("temps") || matches.is_present("status") {
            if matches.is_present("status") {
                println!("\nTemperatures");
            }
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
            let speeds = values_t!(matches, "target_fans_speeds", f64)?;
            if !matches.is_present("auto") {
                proxy.set_auto(false)?;
            }

            for (speed, index) in speeds.into_iter().zip(0..) {
                proxy.set_target_fan_speed(index, speed)?;
            }
        }

        if let Some(config) = matches.value_of("config") {
            proxy.set_config(config.to_owned())?;
        }

        if matches.is_present("auto") {
            proxy.set_auto(true)?;
        } else if matches.is_present("manual") {
            proxy.set_auto(false)?;
        }
    }

    Ok(())
}

fn get_product_name() -> Result<String, std::io::Error> {
    std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
}
