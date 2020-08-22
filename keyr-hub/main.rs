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

pub mod auth;
pub mod cli;
pub mod config;
pub mod database;
pub mod error;

use diesel::prelude::*;

use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, App, HttpServer};
use chrono::{TimeZone, Utc};

use std::path::PathBuf;

use keyr_hubstorage as khs;
use khs::users;

use keyr_types::{KeystrokesStats, Summary, SynchronizeRequest};

use crate::auth::TokenHeader;
use crate::config::HubConfig;
use crate::database::{create_pool, PgPool};
use crate::error::KeyrHubError;

#[post("/commit")]
async fn commit(
    pool : Data<PgPool>,
    tok : TokenHeader,
    request : Json<SynchronizeRequest>,
) -> Result<Json<Summary>, KeyrHubError> {
    let conn = pool.into_inner().get()?;

    let mid = users::identify_user_by_token(&conn, tok.as_token())?;
    let today = Utc.timestamp(request.today, 0);

    Ok(Json(khs::stats::commit(
        &conn,
        mid,
        today,
        &request.staging_area,
    )?))
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

#[get("/view/{name}")]
async fn view_stats(
    pool : Data<PgPool>,
    name : Path<String>,
) -> Result<Json<KeystrokesStats>, KeyrHubError> {
    let conn = pool.into_inner().get()?;

    let res = conn.transaction::<_, KeyrHubError, _>(|| {
        let id = khs::users::find_by_name_in_transaction(&conn, name.clone())?;

        if !khs::users::is_visible_in_transaction(&conn, id)? {
            return Err(KeyrHubError::PrivateData);
        }

        let res = khs::stats::get_keystrokes_stats_in_transaction(&conn, id)?;

        Ok(res)
    })?;

    Ok(Json(res))
}

async fn run() -> anyhow::Result<()> {
    let matches = cli::get_app().get_matches();

    // unwrap is valid since `config_file' is required
    let conf_path = matches.value_of("config_file").unwrap();
    let conf = HubConfig::from_file(&PathBuf::from(conf_path))?;

    let pool = create_pool(&conf.database_url())?;

    khs::migrations::run(&pool.get()?)?;

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(commit)
            .service(revert_initiate)
            .service(revert_terminate)
            .service(revert_cancel)
            .service(view_stats)
    })
    .bind(&format!("{}:{}", conf.http.url, conf.http.port))?
    .run()
    .await?;

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run().await.map_err(|err| {
        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
    })
}
