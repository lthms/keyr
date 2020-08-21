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

use crate::error::{KeyrHubstorageError, Result};
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
    ) -> Result<UserId>
    where Conn : Connection<Backend = Pg> {
        let id = users::table
            .select(users::id)
            .filter(users::id.eq(self.0))
            .get_result::<i32>(conn)
            .optional()?;

        id.map(|x| UserId(x))
            .ok_or(KeyrHubstorageError::UnknownUser)
    }
}

// Create a new user with a given name. Check whether or not the name is
// available before.
pub fn create_user<Conn>(
    conn : &Conn,
    name : String
) -> Result<MaybeUserId>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        create_user_in_transaction(conn, name)
            .map(|x| MaybeUserId(x.0))
    })
}

// Create a new user with a given name. Check whether or not the name is
// available before. This needs to be called from within a transaction.
pub fn create_user_in_transaction<Conn>(
    conn : &Conn,
    name : String
) -> Result<UserId>
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

        Ok(UserId(id))
    } else {
        Err(KeyrHubstorageError::AlreadyUsedNickname(name))
    }
}

// Generate a token for a user identified by a potential id. Returns None if the
// user does not exists.
pub fn generate_token<Conn>(
    conn : &Conn,
    user : MaybeUserId,
) -> Result<Token>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let id = user.validate(conn)?;

        Ok(generate_token_in_transaction(conn, id)?)
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
) -> Result<UserId>
where Conn : Connection<Backend = Pg> {
    let id = tokens::table
        .select(tokens::user_id)
        .filter(tokens::token.eq(&token.0))
        .get_result::<i32>(conn)
        .optional()?;

    id.map(|x| UserId(x))
        .ok_or(KeyrHubstorageError::InvalidToken)
}

// Check whether or not a token is associated by a valid user. User existence
// needs to be asserted again prior to actually using it.
pub fn identify_user_by_token<Conn>(
    conn : &Conn,
    token : &Token,
) -> Result<MaybeUserId>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        identify_user_by_token_in_transaction(
            conn,
            token
        ).map(|x| MaybeUserId(x.0))
    })
}

pub fn freeze_user_in_transaction<Conn>(
    conn : &Conn,
    id : UserId,
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    diesel::update(users::table.find(id.0))
        .set(users::frozen.eq(true))
        .execute(conn)?;

    Ok(())
}

pub fn is_frozen_in_transaction<Conn>(
    conn : &Conn,
    id : UserId,
) -> Result<bool>
where Conn : Connection<Backend = Pg> {
    let res = users::table
        .filter(users::id.eq(id.0))
        .select(users::frozen)
        .get_result::<bool>(conn)?;

    Ok(res)
}

pub fn is_frozen<Conn>(
    conn : &Conn,
    id : MaybeUserId,
) -> Result<bool>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let id = id.validate(conn)?;

        is_frozen_in_transaction(conn, id)
    })
}

pub fn freeze_user<Conn>(
    conn : &Conn,
    mid : MaybeUserId,
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let id = mid.validate(conn)?;

        freeze_user_in_transaction(conn, id)
    })
}

pub fn unfreeze_user_in_transaction<Conn>(
    conn : &Conn,
    id : UserId,
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    diesel::update(users::table.find(id.0))
        .set(users::frozen.eq(false))
        .execute(conn)?;

    Ok(())
}

pub fn unfreeze_user<Conn>(
    conn : &Conn,
    mid : MaybeUserId,
) -> Result<()>
where Conn : Connection<Backend = Pg> {
    conn.transaction(|| {
        let id = mid.validate(conn)?;
        unfreeze_user_in_transaction(conn, id)
    })
}
