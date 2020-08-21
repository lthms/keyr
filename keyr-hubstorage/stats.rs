use chrono::{Utc, NaiveDateTime, DateTime, Timelike, TimeZone};
use diesel::prelude::*;
use diesel::pg::Pg;

use std::collections::HashMap;

use keyr_types::{Summary, KeystrokesStats};

use crate::users::{MaybeUserId, UserId};
use crate::schema::statistics as stats;
use crate::error::{KeyrHubstorageError, Result};

pub fn upsert_keystrokes_count<Conn>(
    conn : &Conn,
    mid : MaybeUserId,
    date : &DateTime<Utc>,
    count : i32,
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let id = mid.validate(conn)?;
        upsert_keystrokes_count_in_transaction(conn, id, &date, count)
    })

}

pub fn upsert_keystrokes_count_in_transaction<Conn>(
    conn : &Conn,
    id : UserId,
    date : &DateTime<Utc>,
    count : i32,
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    if crate::users::is_frozen_in_transaction(conn, id)? {
        return Err(KeyrHubstorageError::FrozenUser);
    }

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

pub fn commit<Conn>(
    conn : &Conn,
    id : MaybeUserId,
    today : DateTime<Utc>,
    sa : &KeystrokesStats,
) -> Result<Summary>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let id = id.validate(conn)?;

        for (t, v) in sa.iter() {
            let date = Utc.timestamp(*t, 0);

            upsert_keystrokes_count_in_transaction(conn, id, &date, *v as i32)?;
        }

        let s = get_summary_in_transaction(conn, id, today)?;

        Ok(s)
    })
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
        .first::<NaiveDateTime>(conn)
        .optional()?
        .unwrap_or(today.naive_utc());

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

pub fn get_keystrokes_stats_in_transaction<Conn>(
    conn : &Conn,
    id : UserId,
) -> Result<KeystrokesStats>
where Conn : Connection<Backend = Pg> {
    let datas = stats::table
        .select((stats::timestamp, stats::count))
        .filter(stats::user_id.eq(id.0))
        .get_results::<(NaiveDateTime, i32)>(conn)?;

    let mut sa = HashMap::new();

    for (t, v) in datas.iter() {
        sa.insert(t.timestamp(), *v as u32);
    }

    Ok(sa)
}

pub fn initiate_revert_in_transaction<Conn>(
    conn : &Conn,
    id : UserId
) -> Result<KeystrokesStats>
where Conn : Connection<Backend = Pg> {
    crate::users::freeze_user_in_transaction(conn, id)?;

    get_keystrokes_stats_in_transaction(conn, id)
}

pub fn initiate_revert<Conn>(
    conn : &Conn,
    id : MaybeUserId
) -> Result<KeystrokesStats>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let id = id.validate(conn)?;
        let res = initiate_revert_in_transaction(conn, id)?;

        Ok(res)
    })
}

pub fn terminate_revert_in_transaction<Conn>(
    conn : &Conn,
    id : UserId
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    diesel::delete(stats::table
                   .filter(stats::user_id.eq(id.0)))
        .execute(conn)?;

    crate::users::unfreeze_user_in_transaction(conn, id)?;

    Ok(())
}

pub fn terminate_revert<Conn>(
    conn : &Conn,
    id : MaybeUserId
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let id = id.validate(conn)?;
        terminate_revert_in_transaction(conn, id)
    })
}
