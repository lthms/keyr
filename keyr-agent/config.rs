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

use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LocalConfig {
    database_path : PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct HubConfig {
    hub_url : String,
    api_token : String,
}

#[derive(Debug, Deserialize)]
pub struct SyncConfig {
    local : LocalConfig,
    hub : HubConfig,
}
