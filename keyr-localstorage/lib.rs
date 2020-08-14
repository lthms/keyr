#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;

use diesel::prelude::*;
use diesel::result::Error;
use diesel::sqlite::SqliteConnection;
use chrono::{DateTime, Utc, Local, Timelike};

mod schema;
pub mod migrations;

use schema::staging_area as sa;

fn get_current_hour() -> DateTime<Utc> {
    Utc::now()
        .with_nanosecond(0).unwrap()
        .with_second(0).unwrap()
        .with_minute(0).unwrap()
}

pub fn get_database() -> ConnectionResult<SqliteConnection> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("keyr").unwrap();

    let path = xdg_dirs.place_config_file("localstorage.sqlite").unwrap();

    SqliteConnection::establish(&path.to_string_lossy())
}

pub fn get_today_count(conn : &SqliteConnection) -> Result<u64, Error> {
    let today = Local::today()
        .and_hms(0, 0, 0)
        .naive_utc();

    let count = sa::table
        .select(diesel::dsl::sum(sa::count))
        .filter(sa::timestamp.ge(today))
        .first::<Option<i64>>(conn)?
        .unwrap_or(0);

    Ok(count as u64)
}

pub fn upsert_hourly_count(conn : &SqliteConnection, count : u32) -> Result<u32, Error> {
    let now = get_current_hour().naive_utc();

    if count != 0 {
        conn.exclusive_transaction(|| {
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
