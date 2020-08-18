/* keyr -- keep track of your keystrokes
 * Copyright (c) 2020 Thomas Letan
 *
 * This file is part of keyr.
 *
 * keyr is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * keyr is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with keyr.  If not, see <https://www.gnu.org/licenses/>.
 */

use clap::{App, SubCommand, Arg, ArgGroup, ArgMatches};

pub enum Output<'a> {
    Json,
    Template(&'a str),
}

pub fn get_app() -> App<'static, 'static> {
    App::new("keyr-sync")
        .version("0.0.0-dev")
        .author("Thomas Letan <lthms@soap.coffee")
        .about("Synchronize your keystrokes locally and remotely")
        .subcommand(
            SubCommand::with_name("stage")
                .about("Fetch the current counter of keyrd and stage it")
        )
        .subcommand(
            SubCommand::with_name("commit")
                .about("Push staging keystrokes to a hub")
                .arg(Arg::with_name("url")
                     .required(true)
                     .index(1))
                .arg(Arg::with_name("token")
                     .required(true)
                     .index(2))
        )
        .subcommand(
            SubCommand::with_name("format")
                .about("Format your keystrokes statistics")
                .args_from_usage(
                    "--template [string] 'A template to output the result'
                     --json 'Output the json as computed'"
                )
                .group(
                    ArgGroup::with_name("output")
                        .args(&["template", "json"])
                )
        )
}

impl<'a> Output<'a> {
    pub fn from_matches(matches : &'a ArgMatches<'static>) -> Self {
        match matches.value_of("template") {
            Some(tmp) => Output::Template(tmp),
            _         => Output::Json,
        }
    }
}
