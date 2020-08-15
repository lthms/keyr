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

use actix_web::{App, HttpServer};
use anyhow::Result;
use keyr_hubstorage as kbs;

async fn run() -> Result<()> {
    let pool = kbs::pool::build("postgres://keyr-hub:@localhost/keyr-hub")?;

    HttpServer::new(move || App::new().data(pool.clone()))
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
