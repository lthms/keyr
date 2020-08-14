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

use std::io::{Result, Read};
use std::os::unix::net::UnixStream;

use keyr_localstorage as kls;

fn keyrd_fetch() -> Result<u32> {
    let mut stream = UnixStream::connect("/tmp/keyrd.socket")?;
    let mut count_buff = [0u8; 4];

    stream.read_exact(&mut count_buff)?;

    Ok(u32::from_le_bytes(count_buff))
}

fn main() -> Result<()> {
    let count = keyrd_fetch()?;

    let conn = kls::get_database().unwrap();
    kls::migrate(&conn).unwrap();
    kls::upsert_current_hour_count(&conn, count).unwrap();

    Ok(())
}
