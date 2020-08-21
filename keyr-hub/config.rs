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

use serde::Deserialize;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub user : String,
    pub password : Option<String>,
    pub url : String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpConfig {
    pub port : isize,
    pub url : String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HubConfig {
    pub http : HttpConfig,
    pub database : DatabaseConfig,
}


impl HubConfig {
    pub fn from_file(path : &Path) -> Result<HubConfig> {
        let res : HubConfig = toml::from_str(
            &std::fs::read_to_string(path)?
        )?;

        Ok(res)
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}",
            self.database.user,
            self.database.password
                .as_ref()
                .map(|x| x.clone())
                .unwrap_or("".to_owned()),
            self.database.url,
        )
    }
}
