#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;

use diesel::prelude::*;
use diesel::result::Error;
use diesel::sqlite::SqliteConnection;
use chrono::{DateTime, Utc, Timelike};

mod schema;
pub mod migrations;

use schema::staging_area;

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

pub fn upsert_hourly_count(conn : &SqliteConnection, count : u32) -> Result<u32, Error> {
    let now = get_current_hour().naive_utc();

    if count != 0 {
        conn.exclusive_transaction(|| {
            let prev = staging_area::table
                .select(staging_area::count)
                .filter(staging_area::timestamp.eq(now))
                .get_result::<i32>(conn)
                .optional()?;

            if let Some(prev) = prev {
                let new_count = prev + count as i32;

                diesel::update(staging_area::table.find(now))
                    .set(staging_area::count.eq(new_count))
                    .execute(conn)?;

                Ok(new_count as u32)
            } else {
                diesel::insert_into(staging_area::table)
                    .values(vec![
                        (staging_area::timestamp.eq(now),
                         staging_area::count.eq(count as i32)),
                    ])
                    .execute(conn)?;

                Ok(count)
            }
        })
    } else {
        Ok(count)
    }

}
