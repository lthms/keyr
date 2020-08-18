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

#[macro_use] extern crate serde_json;
#[macro_use] extern crate anyhow;

use anyhow::Result;

use keyr_agentstorage as kas;

pub mod cli;
pub mod config;
pub mod stage;
pub mod commit;
pub mod format;

use crate::cli::Output;
use crate::config::AgentConfig;

fn main() -> Result<()> {
    let conf = AgentConfig::from_xdg()?;

    let matches = cli::get_app().get_matches();

    let conn = kas::get_database(&conf.local_config()?.database_path)?;
    kas::migrate(&conn)?;

    match matches.subcommand() {
        ("stage", _) => stage::run(&conn)?,
        ("commit", Some(_)) => commit::run(&conn, &conf.hub_config()?)?,
        ("format", Some(m)) => format::run(&conn, &Output::from_matches(m))?,
        _ => println!("nothing to do"),
    }

    Ok(())
}
