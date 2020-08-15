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
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{users, tokens};
use crate::pool::PgPooledConnection;

pub struct UserId(i32);
pub struct MaybeUserId(pub i32);

impl Into<UserId> for i32 {
    fn into(self) -> UserId {
        UserId(self)
    }
}

impl MaybeUserId {
    // Needs to be run in a transaction. Ensure the user exists.
    fn validate(
        &self,
        conn : &PgPooledConnection,
    ) -> Result<Option<UserId>> {
        let id = users::table
            .select(users::id)
            .filter(users::id.eq(self.0))
            .get_result::<i32>(conn)
            .optional()?;

        Ok(id.map(|x| UserId(x)))
    }
}

// Create a new user with a given name. Check whether or not the name is
// available before.
pub fn create_user(
    conn : &PgPooledConnection,
    name : String
) -> Result<Option<MaybeUserId>> {
    conn.transaction(|| {
        let mid = create_user_in_transaction(conn, name)?;

        Ok(mid.map(|x| MaybeUserId(x.0)))
    })
}

// Create a new user with a given name. Check whether or not the name is
// available before. This needs to be called from within a transaction.
fn create_user_in_transaction(
    conn : &PgPooledConnection,
    name : String
) -> Result<Option<UserId>> {
    let prev = users::table
        .select(users::id)
        .filter(users::name.eq(&name))
        .get_results::<i32>(conn)?;

    if prev.len() == 0 {
        let id = diesel::insert_into(users::table)
            .values(vec![
                users::name.eq(&name)
            ])
            .returning(users::id)
            .get_result::<i32>(conn)?;

        Ok(Some(UserId(id)))
    } else {
        Ok(None)
    }
}

// Generate a token for a user identified by a potential id. Returns None if the
// user does not exists.
pub fn generate_token(
    conn : &PgPooledConnection,
    user : MaybeUserId,
) -> Result<Option<Uuid>> {
    conn.transaction(|| {
        if let Some(id) = user.validate(conn)? {
            Ok(Some(generate_token_in_transaction(conn, id)?))
        } else {
            Ok(None)
        }
    })
}

// Generate a token for a user identified by an id whose existence has been
// previously asserted. Needs to be called from within a transaction.
fn generate_token_in_transaction(
    conn : &PgPooledConnection,
    id : UserId,
) -> Result<Uuid> {
    let token = Uuid::new_v4();

    diesel::insert_into(tokens::table)
        .values(vec![
            (tokens::user_id.eq(id.0),
             tokens::token.eq(token.to_simple().to_string()))
        ])
        .execute(conn)?;

    Ok(token)
}
