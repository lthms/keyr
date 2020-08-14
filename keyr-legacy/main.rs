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

mod fscount;

use std::io::Result;
use std::fs::OpenOptions;
use keyr_localstorage as kls;
use fscount::DayFile;

// TODO: Remove this from the repository prior to initial publication
fn main() -> Result<()> {
    let conn = kls::get_database().unwrap();
    kls::migrate(&conn).unwrap();

    for date in fscount::list_days()? {
        let mut file = DayFile::open(&date, OpenOptions::new().read(true).clone())?;

        file.read_global_count()?;

        loop {
            if let Some((key, count)) = file.read_entry()? {
                let (h, _) = fscount::parse_key(&key).unwrap();

                kls::upsert_hour_count(&conn, date, h, count).unwrap();
            } else {
                break;
            }
        }
    }

    println!(
        "migration completed. {} keystrokes in total, {} today",
        kls::get_global_count(&conn).unwrap(),
        kls::get_today_count(&conn).unwrap(),
    );

    Ok(())
}
