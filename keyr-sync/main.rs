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

use std::io::Read;
use std::os::unix::net::UnixStream;

use anyhow::Result;
use clap::{App, Arg, SubCommand};
use reqwest::blocking::Client;
use chrono::{Utc, TimeZone, Local};

use keyr_localstorage as kls;
use kls::SqliteConnection;

use keyr_types::{StagingArea, SynchronizeRequest, Summary};

fn get_app() -> App<'static, 'static> {
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
}

fn keyrd_fetch() -> Result<u32> {
    let mut stream = UnixStream::connect("/tmp/keyrd.socket")?;
    let mut count_buff = [0u8; 4];

    stream.read_exact(&mut count_buff)?;

    Ok(u32::from_le_bytes(count_buff))
}

fn stage() -> Result<()> {
    let conn = kls::get_database()?;
    kls::migrate(&conn)?;

    let count = keyrd_fetch()?;
    kls::upsert_current_hour_count(&conn, count)?;

    Ok(())
}

fn commit_inner(
    conn : &SqliteConnection,
    url : &str,
    token : &str,
    sa : StagingArea
) -> Result<()> {
    let client = Client::new();

    let today = Local::today()
        .and_hms(0, 0, 0)
        .naive_utc();

    let req = SynchronizeRequest {
        staging_area : sa,
        today : today.timestamp()
    };

    let resp = client.post(url)
        .json(&req)
        .header("Keyr-Token", token)
        .send()?;

    if resp.status().is_success() {
        let resp : Summary = resp.json()?;

        kls::set_summary_in_transaction(
            &conn,
            Utc.timestamp(resp.oldest_timestamp, 0),
            resp.global_count,
            Utc.timestamp(resp.today_timestamp, 0),
            resp.today_count,
        )?;

        Ok(())
    } else {
        panic!() // FIXME
    }
}

fn commit(url : &str, token : &str) -> Result<()> {
    let conn = kls::get_database()?;
    kls::migrate(&conn)?;

    kls::commit(&conn, |sa| commit_inner(&conn, url, token, sa))?;

    Ok(())
}

fn main() -> Result<()> {
    let matches = get_app().get_matches();

    match matches.subcommand() {
        ("stage", _) => stage()?,
        ("commit", Some(m)) => {
            let url = m.value_of("url").unwrap();
            let token = m.value_of("token").unwrap();

            commit(url, token)?;
        },
        _ => println!("nothing to do"),
    }

    Ok(())
}
