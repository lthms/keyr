use chrono::{Utc, DateTime, Timelike};
use diesel::prelude::*;
use diesel::pg::Pg;

use crate::users::{MaybeUserId, UserId};
use crate::schema::statistics as stats;
use crate::error::Result;

pub fn upsert_keystrokes_count<Conn>(
    conn : &Conn,
    mid : MaybeUserId,
    date : &DateTime<Utc>,
    count : i32,
) -> Result<bool>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        if let Some(id) = mid.validate(conn)? {
            upsert_keystrokes_count_in_transaction(conn, id, &date, count)?;
            Ok(true)
        } else {
            Ok(false)
        }
    })

}

pub(crate) fn upsert_keystrokes_count_in_transaction<Conn>(
    conn : &Conn,
    id : UserId,
    date : &DateTime<Utc>,
    count : i32,
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    let date = date
        .with_nanosecond(0).unwrap()
        .with_second(0).unwrap()
        .with_minute(0).unwrap()
        .naive_utc();

    let prev = stats::table
        .select((stats::id, stats::count))
        .filter(stats::timestamp.eq(&date))
        .filter(stats::user_id.eq(id.0))
        .get_result::<(i32, i32)>(conn)
        .optional()?;

    match prev {
        Some((id, prev_count)) => {
            diesel::update(stats::table.find(id))
                .set(stats::count.eq(prev_count + count))
                .execute(conn)?;
        },
        None => {
            diesel::insert_into(stats::table)
                .values(vec![
                    (stats::timestamp.eq(&date),
                     stats::count.eq(count),
                     stats::user_id.eq(id.0))
                ])
                .execute(conn)?;
        }
    }

    Ok(())
}
