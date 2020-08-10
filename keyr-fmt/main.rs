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
 * Foobar is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Foobar.  If not, see <https://www.gnu.org/licenses/>.
 */

#[macro_use] extern crate serde_json;

use chrono::{Date, Utc};
use std::io::Result;
use std::fs::OpenOptions;
use serde_json::Value;
use keyr::{CounterFile, DayFile, GlobalFile, EntryLoc};

fn options() -> OpenOptions {
    OpenOptions::new()
        .read(true)
        .clone()
}

fn day_summary(date : &Date<Utc>) -> Result<Value> {
    let mut day_file = DayFile::open(date, options())?;
    let mut buff = vec![];

    let count = day_file.read_global_count()?;

    loop {
        if let Some((key, count)) = day_file.read_entry(EntryLoc::Next)? {
            if let Some((hour, minute)) = keyr::parse_key(&key) {
                let time = date.and_hms(hour, minute, 0);

                buff.push(json!({
                    "time": time.to_rfc2822(),
                    "count": count
                }))

            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(json!({
        "count": count,
        "log": buff,
    }))
}

fn main() -> Result<()> {
    let mut global = GlobalFile::open(options())?;

    let days_summaries = keyr::list_days()?
        .iter()
        .map(day_summary)
        .collect::<Result<Vec<_>>>()?;

    let res = json!({
        "count": global.read_global_count()?,
        "days": days_summaries,
    });

    println!("{}", res.to_string());

    Ok(())
}
