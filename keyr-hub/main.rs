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
pub mod config;
pub mod cli;

use actix_web::{App, HttpServer, post};
use actix_web::web::{Json, Data};
use chrono::{Utc, TimeZone};

use std::path::PathBuf;

use keyr_hubstorage as khs;
use khs::users::identify_user_by_token;

use keyr_types::{SynchronizeRequest, Summary};

use crate::error::KeyrHubError;
use crate::database::{PgPool, create_pool};
use crate::auth::TokenHeader;
use crate::config::HubConfig;

#[post("/sync")]
async fn sync(
    pool : Data<PgPool>,
    tok : TokenHeader,
    request : Json<SynchronizeRequest>
) -> Result<Json<Summary>, KeyrHubError> {
    let conn = pool.into_inner().get()?;

    if let Some(mid) = identify_user_by_token(&conn, tok.as_token())? {
        let today = Utc.timestamp(request.today, 0);

        // FIXME
        Ok(
            Json(
                khs::stats::sync(&conn, mid, today, &request.staging_area)?.unwrap()
            )
        )
    } else {
        Err(KeyrHubError::IncorrectToken)
    }
}

async fn run() -> anyhow::Result<()> {
    let matches = cli::get_app().get_matches();

    // unwrap is valid since `config_file' is required
    let conf_path = matches.value_of("config_file").unwrap();
    let conf = HubConfig::from_file(&PathBuf::from(conf_path))?;

    let pool = create_pool(&conf.database_url())?;

    khs::migrations::run(&pool.get()?)?;

    HttpServer::new(
        move || App::new()
            .data(pool.clone())
            .service(sync))
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
