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

use actix_web::{
    App, HttpServer, FromRequest, HttpRequest, get,

    dev::Payload,
    error::ResponseError,
    http::StatusCode,
    web::Data,
};
use futures::future::{Ready, ok, err};
use thiserror::Error;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub type PgConnectionManager = ConnectionManager<PgConnection>;
pub type PgPool = Pool<PgConnectionManager>;
pub type PgPooledConnection = PooledConnection<PgConnectionManager>;

use keyr_hubstorage as kbs;
use kbs::users::Token;
use kbs::error::KeyrHubstorageError;

#[derive(Error, Debug)]
pub enum KeyrHubError {
    #[error("Client trying to access a protected route without setting Keyr-Token header")]
    MissingKeyrTokenHeader,
    #[error("Client trying to access a protected route with an incorrect token")]
    IncorrectToken,
    #[error(transparent)]
    Storage(#[from] KeyrHubstorageError),
    #[error(transparent)]
    Pool(#[from] r2d2::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl ResponseError for KeyrHubError {
    fn status_code(&self) -> StatusCode {
        match self {
            KeyrHubError::MissingKeyrTokenHeader => StatusCode::UNAUTHORIZED,
            KeyrHubError::IncorrectToken => StatusCode::UNAUTHORIZED,
            KeyrHubError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            KeyrHubError::Pool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            KeyrHubError::IO(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub struct TokenHeader(Token);

impl FromRequest for TokenHeader {
    type Config = ();
    type Error = KeyrHubError;
    type Future = Ready<Result<TokenHeader, KeyrHubError>>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        if let Some(token) = req.headers().get("keyr-token") {
            if let Ok(token) = token.to_str() {
                ok(TokenHeader(Token(token.to_owned())))
            } else {
                err(KeyrHubError::IncorrectToken)
            }
        } else {
            err(KeyrHubError::MissingKeyrTokenHeader)
        }
    }
}

#[get("/whoami")]
async fn index(pool : Data<PgPool>, tok : TokenHeader) -> Result<String, KeyrHubError> {
    let conn = pool.into_inner().get()?;
    if let Some(_id) = kbs::users::identify_user_by_token(&conn, tok.0)? {
        Ok("yay".to_owned())
    } else {
        Err(KeyrHubError::IncorrectToken)
    }
}

async fn run() -> Result<(), KeyrHubError> {
    let pool = Pool::builder()
        .build(PgConnectionManager::new("postgres://keyr-hub:@localhost/keyr-hub"))?;

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
