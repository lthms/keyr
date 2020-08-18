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

use diesel::prelude::*;
use diesel::pg::Pg;
use uuid::Uuid;

use crate::error::Result;
use crate::schema::{users, tokens};

#[derive(Copy, Clone)]
pub struct UserId(pub i32);

#[derive(Copy, Clone)]
pub struct MaybeUserId(pub i32);

#[derive(Clone)]
pub struct Token(pub String);

impl Into<UserId> for i32 {
    fn into(self) -> UserId {
        UserId(self)
    }
}

impl MaybeUserId {
    // Needs to be run in a transaction. Ensure the user exists.
    pub fn validate<Conn>(
        &self,
        conn : &Conn,
    ) -> Result<Option<UserId>>
    where Conn : Connection<Backend = Pg> {
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
pub fn create_user<Conn>(
    conn : &Conn,
    name : String
) -> Result<Option<MaybeUserId>>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let mid = create_user_in_transaction(conn, name)?;

        Ok(mid.map(|x| MaybeUserId(x.0)))
    })
}

// Create a new user with a given name. Check whether or not the name is
// available before. This needs to be called from within a transaction.
pub fn create_user_in_transaction<Conn>(
    conn : &Conn,
    name : String
) -> Result<Option<UserId>>
where Conn : Connection<Backend = Pg> {
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
pub fn generate_token<Conn>(
    conn : &Conn,
    user : MaybeUserId,
) -> Result<Option<Token>>
where Conn : Connection<Backend = Pg> {
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
pub fn generate_token_in_transaction<Conn>(
    conn : &Conn,
    id : UserId,
) -> Result<Token>
where Conn : Connection<Backend = Pg> {
    let token = Uuid::new_v4().to_simple().to_string();

    diesel::insert_into(tokens::table)
        .values(vec![
            (tokens::user_id.eq(id.0),
             tokens::token.eq(&token))
        ])
        .execute(conn)?;

    Ok(Token(token))
}

// Check whether or not a token is associated by a valid user. Needs to be
// called from within a transaction.
pub fn identify_user_by_token_in_transaction<Conn>(
    conn : &Conn,
    token : &Token,
) -> Result<Option<UserId>>
where Conn : Connection<Backend = Pg> {
    let id = tokens::table
        .select(tokens::user_id)
        .filter(tokens::token.eq(&token.0))
        .get_result::<i32>(conn)
        .optional()?;

    Ok(id.map(|x| UserId(x)))
}

// Check whether or not a token is associated by a valid user. User existence
// needs to be asserted again prior to actually using it.
pub fn identify_user_by_token<Conn>(
    conn : &Conn,
    token : &Token,
) -> Result<Option<MaybeUserId>>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let id = identify_user_by_token_in_transaction(
            conn,
            token
        )?;

        Ok(id.map(|x| MaybeUserId(x.0)))
    })
}
