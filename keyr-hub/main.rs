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
use khs::users;

use keyr_types::{SynchronizeRequest, KeystrokesStats, Summary};

use crate::error::KeyrHubError;
use crate::database::{PgPool, create_pool};
use crate::auth::TokenHeader;
use crate::config::HubConfig;

#[post("/commit")]
async fn commit(
    pool : Data<PgPool>,
    tok : TokenHeader,
    request : Json<SynchronizeRequest>
) -> Result<Json<Summary>, KeyrHubError> {
    let conn = pool.into_inner().get()?;

    let mid = users::identify_user_by_token(&conn, tok.as_token())?;
    let today = Utc.timestamp(request.today, 0);

    // FIXME
    Ok(
        Json(
            khs::stats::sync(&conn, mid, today, &request.staging_area)?
        )
    )
}

#[post("/revert/initiate")]
async fn revert_initiate(
    pool : Data<PgPool>,
    tok : TokenHeader,
) -> Result<Json<KeystrokesStats>, KeyrHubError> {
    let conn = pool.into_inner().get()?;

    let mid = users::identify_user_by_token(&conn, tok.as_token())?;
    let res = khs::stats::initiate_revert(&conn, mid)?;

    Ok(Json(res))
}

#[post("/revert/terminate")]
async fn revert_terminate(
    pool : Data<PgPool>,
    tok : TokenHeader,
) -> Result<Json<()>, KeyrHubError> {
    let conn = pool.into_inner().get()?;

    let mid = users::identify_user_by_token(&conn, tok.as_token())?;
    let res = khs::stats::terminate_revert(&conn, mid)?;

    Ok(Json(res))
}

#[post("/revert/cancel")]
async fn revert_cancel(
    pool : Data<PgPool>,
    tok : TokenHeader,
) -> Result<Json<()>, KeyrHubError> {
    let conn = pool.into_inner().get()?;

    let mid = users::identify_user_by_token(&conn, tok.as_token())?;
    khs::users::unfreeze_user(&conn, mid)?;

    Ok(Json(()))
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
            .service(commit)
            .service(revert_initiate)
            .service(revert_terminate)
            .service(revert_cancel)
    )
        .bind(&format!("{}:{}", conf.http.url, conf.http.port))?
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
