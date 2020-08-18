use chrono::{Utc, NaiveDateTime, DateTime, Timelike, TimeZone};
use diesel::prelude::*;
use diesel::pg::Pg;

use keyr_types::{Summary, StagingArea};

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

pub fn upsert_keystrokes_count_in_transaction<Conn>(
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

pub fn sync<Conn>(
    conn : &Conn,
    id : MaybeUserId,
    today : DateTime<Utc>,
    sa : &StagingArea,
) -> Result<Option<Summary>>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        if let Some(id) = id.validate(conn)? {
            commit_staging_area_in_transaction(conn, id, sa)?;
            let s = get_summary_in_transaction(conn, id, today)?;

            Ok(Some(s))
        } else {
            Ok(None)
        }
    })
}

pub fn commit_staging_area_in_transaction<Conn>(
    conn : &Conn,
    id : UserId,
    sa : &StagingArea,
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    for (t, v) in sa.iter() {
        let date = Utc.timestamp(*t, 0);

        upsert_keystrokes_count_in_transaction(conn, id, &date, *v as i32)?;
    }

    Ok(())
}


pub fn get_summary_in_transaction<Conn>(
    conn : &Conn,
    id : UserId,
    today : DateTime<Utc>,
) -> Result<Summary>
where Conn : Connection<Backend = Pg> {
    let oldest_entry = stats::table
        .select(stats::timestamp)
        .filter(stats::user_id.eq(id.0))
        .order(stats::timestamp.asc())
        .first::<NaiveDateTime>(conn)?;

    let today_count = stats::table
        .select(diesel::dsl::sum(stats::count))
        .filter(stats::timestamp.ge(today.naive_utc()))
        .first::<Option<i64>>(conn)?
        .unwrap_or(0);

    let global_count = stats::table
        .select(diesel::dsl::sum(stats::count))
        .first::<Option<i64>>(conn)?
        .unwrap_or(0);

    Ok(Summary {
        oldest_timestamp : oldest_entry.timestamp(),
        global_count : global_count as u64,
        today_count : today_count as u64,
        today_timestamp : today.naive_utc().timestamp(),
    })
}
