/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use clap::{crate_version, App, AppSettings, Arg, SubCommand};

pub fn get_app() -> App<'static, 'static> {
    App::new("fancy")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("set")
                .about("Set a value")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    Arg::with_name("target_fans_speeds")
                        .help("Set the fans speeds")
                        .short("f")
                        .long("fans-speeds")
                        .takes_value(true)
                        .multiple(true)
                        .value_name("TARGET_FAN_SPEEDS")
                        .conflicts_with("auto"),
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
                        .long("auto")
                        .conflicts_with("target_fans_speeds"),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get a value")
                .setting(AppSettings::SubcommandRequiredElseHelp)
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
                        .help("Filter to get only the recommended ones"),
                ),
        )
}
