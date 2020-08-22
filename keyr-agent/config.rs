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
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct LocalConfig {
    pub database_path : PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HubConfig {
    pub hub_url : String,
    pub api_token : String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentConfig {
    local : Option<LocalConfig>,
    hub : Option<HubConfig>,
}

impl AgentConfig {
    pub fn default() -> AgentConfig {
        AgentConfig {
            local : None,
            hub : None,
        }
    }

    pub fn from_xdg() -> Result<AgentConfig> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("keyr")?;
        let path = xdg_dirs.place_config_file("keyr.toml")?;

        if path.exists() {
            AgentConfig::from_file(&path)
        } else {
            Ok(AgentConfig::default())
        }
    }

    pub fn from_file(path : &Path) -> Result<AgentConfig> {
        let mut res : AgentConfig =
            toml::from_str(&std::fs::read_to_string(path)?)?;

        if let Some(ref local) = res.local {
            if local.database_path.is_relative() {
                res.local = Some(LocalConfig {
                    database_path : path.join(local.database_path.clone()),
                })
            }
        }

        Ok(res)
    }

    pub fn local_config(&self) -> Result<LocalConfig> {
        match &self.local {
            Some(local) => Ok(local.clone()),
            None => {
                let xdg_dirs = xdg::BaseDirectories::with_prefix("keyr")?;
                let path = xdg_dirs.place_config_file("localstorage.sqlite")?;

                Ok(LocalConfig {
                    database_path : path.to_owned(),
                })
            }
        }
    }

    pub fn hub_config(&self) -> Result<HubConfig> {
        match &self.hub {
            Some(hub) => Ok(hub.clone()),
            None => bail!("Missing keyr-hub configuration."),
        }
    }
}
