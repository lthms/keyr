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

use anyhow::Result;
use chrono::{Local, TimeZone, Utc};
use reqwest::blocking::Client;

use kas::SqliteConnection;
use keyr_agentstorage as kas;
use keyr_types::{KeystrokesStats, Summary, SynchronizeRequest};

use crate::config::HubConfig;

fn commit_inner(
    conn : &SqliteConnection,
    url : &str,
    token : &str,
    sa : KeystrokesStats,
) -> Result<()> {
    let client = Client::new();

    let today = Local::today().and_hms(0, 0, 0).naive_utc();

    let req = SynchronizeRequest {
        staging_area : sa,
        today : today.timestamp(),
    };

    let resp = client
        .post(&format!("{}/commit", url))
        .json(&req)
        .header("Keyr-Token", token)
        .send()?;

    if resp.status().is_success() {
        let resp : Summary = resp.json()?;

        kas::set_summary_in_transaction(
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

pub fn run(conn : &SqliteConnection, hub : &HubConfig) -> Result<()> {
    kas::commit(&conn, |sa| {
        commit_inner(&conn, &hub.hub_url, &hub.api_token, sa)
    })?;

    Ok(())
}
