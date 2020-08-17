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

pub mod error;
pub mod database;
pub mod auth;

use actix_web::{App, HttpServer, get};
use actix_web::web::Data;

use keyr_hubstorage as kbs;
use kbs::users::identify_user_by_token;

use crate::error::KeyrHubError;
use crate::database::{PgPool, create_pool};
use crate::auth::TokenHeader;

#[get("/whoami")]
async fn index(pool : Data<PgPool>, tok : TokenHeader) -> Result<String, KeyrHubError> {
    let conn = pool.into_inner().get()?;

    if let Some(_id) = identify_user_by_token(&conn, tok.as_token())? {
        Ok("yay".to_owned())
    } else {
        Err(KeyrHubError::IncorrectToken)
    }
}

async fn run() -> Result<(), KeyrHubError> {
    let pool = create_pool("postgres://keyr-hub:@localhost/keyr-hub")?;

    kbs::migrations::run(&pool.get()?)?;

    HttpServer::new(
        move || App::new()
            .data(pool.clone())
            .service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await?;

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run().await
        .map_err(
            |err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
        )
}
