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

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;

use diesel::prelude::*;
use diesel::result::Error;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::RunMigrationsError;
use chrono::{DateTime, Utc, Local, Timelike, Date, TimeZone};
use std::time::Duration;

mod schema;
mod migrations;

use schema::staging_area as sa;
use schema::summary;

pub fn get_database() -> ConnectionResult<SqliteConnection> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("keyr").unwrap();

    let path = xdg_dirs.place_config_file("localstorage.sqlite").unwrap();

    SqliteConnection::establish(&path.to_string_lossy())
}

// Helper to perform atomic transactions concurrently. We try as many time as
// necessary.
fn transaction_retry<A, F>(conn : &SqliteConnection, f : &F) -> Result<A, Error>
where F : Fn() -> Result<A, Error> {
    loop {
        match conn.exclusive_transaction(f) {
            Err(err) => {
                if err.to_string() == "database is locked" {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }

                break Err(err)
            },
            res => break res,
        }
    }
}

pub fn get_today_count(conn : &SqliteConnection) -> Result<u64, Error> {
    let today = Local::today()
        .and_hms(0, 0, 0)
        .naive_utc();

    transaction_retry(conn, &|| {
        let staging_count = sa::table
            .select(diesel::dsl::sum(sa::count))
            .filter(sa::timestamp.ge(today))
            .first::<Option<i64>>(conn)?
            .unwrap_or(0);

        let summary_count = summary::table
            .select(summary::count)
            .filter(summary::since.eq(today))
            .get_result::<i64>(conn)
            .optional()?
            .unwrap_or(0);

        Ok(staging_count as u64 + summary_count as u64)

    })
}

pub fn get_global_count(conn : &SqliteConnection) -> Result<u64, Error> {
    transaction_retry(conn, &|| {
        let staging_count = sa::table
            .select(diesel::dsl::sum(sa::count))
            .first::<Option<i64>>(conn)?
            .unwrap_or(0);

        let summary_count = summary::table
            .select(summary::count)
            .order(summary::since.asc())
            .first::<i64>(conn)
            .optional()?
            .unwrap_or(0);

        Ok((staging_count + summary_count) as u64)
    })
}

pub fn set_summary(
    conn : &SqliteConnection,
    oldest : DateTime<Utc>,
    global_count : u64,
    today : DateTime<Utc>,
    today_count : u64
) -> Result<(), Error> {
    transaction_retry(conn, &|| {
        diesel::delete(summary::table)
            .execute(conn)?;

        diesel::insert_into(summary::table)
            .values(vec![
                (summary::since.eq(oldest.naive_utc()),
                 summary::count.eq(global_count as i64)),
            ])
            .execute(conn)?;

        diesel::insert_into(summary::table)
            .values(vec![
                (summary::since.eq(today.naive_utc()),
                 summary::count.eq(today_count as i64)),
            ])
            .execute(conn)?;

        Ok(())
    })
}

pub fn upsert_current_hour_count(conn : &SqliteConnection, count : u32) -> Result<u32, Error> {
    let now = Utc::now()
        .with_nanosecond(0).unwrap()
        .with_second(0).unwrap()
        .with_minute(0).unwrap();

    upsert_hour_count(conn, now.date(), now.hour(), count)
}

pub fn upsert_hour_count<Tz>(
    conn : &SqliteConnection,
    dt : Date<Tz>,
    hour : u32,
    count : u32,
) -> Result<u32, Error>
where Tz : TimeZone {
    let now = dt.and_hms(hour, 0, 0).naive_utc();

    if count != 0 {
        transaction_retry(conn, &|| {
            let prev = sa::table
                .select(sa::count)
                .filter(sa::timestamp.eq(now))
                .get_result::<i32>(conn)
                .optional()?;

            if let Some(prev) = prev {
                let new_count = prev + count as i32;

                diesel::update(sa::table.find(now))
                    .set(sa::count.eq(new_count))
                    .execute(conn)?;

                Ok(new_count as u32)
            } else {
                diesel::insert_into(sa::table)
                    .values(vec![
                        (sa::timestamp.eq(now),
                         sa::count.eq(count as i32)),
                    ])
                    .execute(conn)?;

                Ok(count)
            }
        })
    } else {
        Ok(count)
    }
}

pub fn migrate(conn : &SqliteConnection) -> Result<(), Error> {
    transaction_retry(conn, &|| {
        migrations::run(conn).map_err(|err| {
            match err {
                RunMigrationsError::QueryError(err) => err,
                _ => panic!("FIXME")
            }
        })
    })
}
