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

use chrono::{Date, Utc};
use chrono::offset::TimeZone;
use std::fs::{OpenOptions, File};
use std::path::{Path, PathBuf};
use std::io::{Result, Read};

pub struct DayFile(File);

impl DayFile {
    pub fn read_global_count(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.0.read_exact(&mut buf)?;

        Ok(u32::from_le_bytes(buf))
    }

    pub fn open(date : &Date<Utc>, opts : OpenOptions) -> Result<DayFile> {
        let path = data_path(&date.format("%Y%m%d").to_string())?;

        Ok(DayFile(opts.open(path)?))
    }

    fn read_key(&mut self) -> Result<String> {
        let mut buf = [0u8; 4];
        self.0.read_exact(&mut buf)?;

        // FIXME
        Ok(String::from_utf8(Vec::from(buf.as_ref())).unwrap())
    }

    fn read_count(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.0.read_exact(&mut buf)?;

        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_entry(&mut self) -> Result<Option<(String, u32)>> {
        match self.read_key() {
            Ok(key) => { // we have been able to read a key
                         // if we cannot read a count, there is an error in our
                         // file
                Ok(Some((key, self.read_count()?)))
            }
            _ => { // we could not read a key, so we assume there is no
                   // entry to read
                Ok(None)
            },
        }
    }
}

fn data_path(name : &str) -> Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("keyr").unwrap();

    xdg_dirs.place_config_file(name)
}

fn parse_filename(p : &Path) -> Option<Date<Utc>> {
    let re_file = regex::Regex::new(r"^(\d{4})(\d{2})(\d{2})$").unwrap();

    let filename = p.file_name()?.to_string_lossy();

    let cap = re_file.captures_iter(&filename).next()?;

    let year = cap[1].parse::<i32>().ok()?;
    let month = cap[2].parse::<u32>().ok()?;
    let day = cap[3].parse::<u32>().ok()?;

    Utc.ymd_opt(year as i32, month, day).single()
}

pub fn parse_key(key : &str) -> Option<(u32, u32)> {
    let re_file = regex::Regex::new(r"^(\d{2})(\d{2})$").unwrap();

    let cap = re_file.captures_iter(key).next()?;

    let hour = cap[1].parse::<u32>().ok()?;
    let minute = cap[2].parse::<u32>().ok()?;

    Some((hour, minute))
}

pub fn list_days() -> Result<Vec<Date<Utc>>> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("keyr").unwrap();
    let mut res = vec![];

    for candidate in xdg_dirs.get_config_home().read_dir()? {
        let candidate = candidate?.path();

        if candidate.is_file() {
            if let Some(x) = parse_filename(&candidate) {
                res.push(x);
            }
        }
    }

    Ok(res)
}
