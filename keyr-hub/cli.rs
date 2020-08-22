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

use clap::{App, Arg};

pub fn get_app() -> App<'static, 'static> {
    App::new("keyr-hub")
        .version("0.0.0-dev")
        .author("Thomas Letan <lthms@soap.coffee")
        .about("A hub to synchronize your keystrokes between several computers")
        .arg(
            Arg::with_name("config_file")
                .help("A path to a TOML file")
                .long("config-file")
                .value_name("FILE")
                .required(true),
        )
}
