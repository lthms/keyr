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

pub struct UserId(pub i32);

use crate::schema::tokens;
use crate::pool::PgPooledConnection;

pub fn generate_token(
    conn : &PgPooledConnection,
    user : UserId,
) -> Result<Uuid> {
    let token = Uuid::new_v4();

    diesel::insert_into(tokens::table)
        .values(vec![
            (tokens::user_id.eq(user.0),
             tokens::token.eq(token.to_simple().to_string()))
        ])
        .execute(conn)?;

    Ok(token)
}
