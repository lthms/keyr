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

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use chrono::{Date, DateTime, Local, NaiveDateTime, TimeZone, Timelike, Utc};
use diesel::prelude::*;
use diesel::result::Error;
pub use diesel::sqlite::SqliteConnection;
use diesel_migrations::RunMigrationsError;

use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;
use std::time::Duration;

mod migrations;
mod schema;

use schema::staging_area as sa;
use schema::summary;

use keyr_types::KeystrokesStats;

pub fn get_database(path : &Path) -> ConnectionResult<SqliteConnection> {
    SqliteConnection::establish(&path.to_string_lossy())
}

// Helper to perform atomic transactions concurrently. We try as many time as
// necessary.
pub fn transaction_retry<A, E, F>(
    conn : &SqliteConnection,
    f : &F,
) -> Result<A, E>
where
    E : From<Error> + Display,
    F : Fn() -> Result<A, E>,
{
    loop {
        match conn.exclusive_transaction(f) {
            Err(err) => {
                if err.to_string() == "database is locked" {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }

                break Err(err);
            }
            res => break res,
        }
    }
}

pub fn get_today_count(conn : &SqliteConnection) -> Result<u64, Error> {
    let today = Local::today().and_hms(0, 0, 0).naive_utc();

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

pub fn drop_summary(conn : &SqliteConnection) -> Result<(), Error> {
    diesel::delete(summary::table).execute(conn)?;

    Ok(())
}

pub fn set_summary_in_transaction(
    conn : &SqliteConnection,
    oldest : DateTime<Utc>,
    global_count : u64,
    today : DateTime<Utc>,
    today_count : u64,
) -> Result<(), Error> {
    diesel::delete(summary::table).execute(conn)?;

    diesel::insert_into(summary::table)
        .values(vec![(
            summary::since.eq(oldest.naive_utc()),
            summary::count.eq(global_count as i64),
        )])
        .execute(conn)?;

    diesel::insert_into(summary::table)
        .values(vec![(
            summary::since.eq(today.naive_utc()),
            summary::count.eq(today_count as i64),
        )])
        .execute(conn)?;

    Ok(())
}

pub fn upsert_current_hour_count(
    conn : &SqliteConnection,
    count : u32,
) -> Result<u32, Error> {
    let now = Utc::now()
        .with_nanosecond(0)
        .unwrap()
        .with_second(0)
        .unwrap()
        .with_minute(0)
        .unwrap();

    upsert_hour_count(conn, now.date(), now.hour(), count)
}

pub fn upsert_hour_count_in_transaction<Tz>(
    conn : &SqliteConnection,
    dt : Date<Tz>,
    hour : u32,
    count : u32,
) -> Result<u32, Error>
where
    Tz : TimeZone,
{
    let now = dt.and_hms(hour, 0, 0).naive_utc();

    if count != 0 {
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
                .values(vec![(
                    sa::timestamp.eq(now),
                    sa::count.eq(count as i32),
                )])
                .execute(conn)?;

            Ok(count)
        }
    } else {
        Ok(count)
    }
}

pub fn upsert_hour_count<Tz>(
    conn : &SqliteConnection,
    dt : Date<Tz>,
    hour : u32,
    count : u32,
) -> Result<u32, Error>
where
    Tz : TimeZone,
{
    transaction_retry(conn, &|| {
        upsert_hour_count_in_transaction(conn, dt.clone(), hour, count)
    })
}

pub fn migrate(conn : &SqliteConnection) -> Result<(), Error> {
    transaction_retry(conn, &|| {
        migrations::run(conn).map_err(|err| match err {
            RunMigrationsError::QueryError(err) => err,
            _ => panic!("FIXME"),
        })
    })
}

fn get_staging_area_in_transaction(
    conn : &SqliteConnection,
) -> Result<KeystrokesStats, Error> {
    let datas = sa::table
        .select((sa::timestamp, sa::count))
        .get_results::<(NaiveDateTime, i32)>(conn)?;

    let mut sa = HashMap::new();

    for (t, v) in datas.iter() {
        sa.insert(t.timestamp(), *v as u32);
    }

    Ok(sa)
}

fn drop_staging_area_in_transaction(
    conn : &SqliteConnection,
) -> Result<(), Error> {
    diesel::delete(sa::table).execute(conn)?;

    Ok(())
}

pub fn commit<A, E, K>(conn : &SqliteConnection, k : K) -> Result<A, Error>
where
    K : Fn(KeystrokesStats) -> Result<A, E>,
{
    transaction_retry(conn, &|| {
        let sa = get_staging_area_in_transaction(conn)?;

        match k(sa) {
            Ok(res) => {
                drop_staging_area_in_transaction(conn)?;
                Ok(res)
            }
            Err(_) => {
                panic!() // FIXME
            }
        }
    })
}
