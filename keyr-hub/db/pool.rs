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
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

embed_migrations!();

pub type PgConnectionManager = ConnectionManager<PgConnection>;
pub type PgPool = Pool<PgConnectionManager>;

// Create a pool of Postgresql connections. Run the migrations if necessary.
pub fn build() -> Result<PgPool> {
    let pool = Pool::builder()
        .build(
            PgConnectionManager::new(
                "postgres://keyr-hub:@localhost/keyr-hub"
            )
        )?;

    embedded_migrations::run(&pool.get()?)?;

    Ok(pool)
}
