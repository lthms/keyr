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
use chrono::{TimeZone, Timelike, Utc};
use reqwest::blocking::Client;

use kas::SqliteConnection;
use keyr_agentstorage as kas;
use keyr_types::KeystrokesStats;

use crate::config::HubConfig;

pub fn run(conn : &SqliteConnection, hub : &HubConfig) -> Result<()> {
    let client = Client::new();

    kas::transaction_retry(&conn, &|| {
        let resp = client
            .post(&format!("{}/revert/initiate", &hub.hub_url))
            .header("Keyr-Token", &hub.api_token)
            .send()?;

        if resp.status().is_success() {
            let resp : KeystrokesStats = resp.json()?;

            for (t, v) in resp {
                let d = Utc.timestamp(t, 0);
                kas::upsert_hour_count_in_transaction(
                    conn,
                    d.date(),
                    d.hour(),
                    v,
                )?;
            }

            kas::drop_summary(&conn)?;

            let resp = client
                .post(&format!("{}/revert/terminate", &hub.hub_url))
                .header("Keyr-Token", &hub.api_token)
                .send()?;

            if resp.status().is_success() {
                Ok(())
            } else {
                panic!() // FIXME
            }
        } else {
            panic!() // FIXME
        }
    })
}
