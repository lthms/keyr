use chrono::Utc;
use std::io::{Result, Read};
use std::os::unix::net::UnixStream;
use std::fs::OpenOptions;

use keyr::{CounterFile, DayFile, GlobalFile, EntryLoc};

fn open_data_options() -> OpenOptions {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .clone()
}

fn keyrd_fetch() -> Result<u32> {
    let mut stream = UnixStream::connect("/tmp/keyrd.socket")?;
    let mut count_buff = [0u8; 4];

    stream.read_exact(&mut count_buff)?;

    Ok(u32::from_le_bytes(count_buff))
}

fn update_global_count(count : u32) -> Result<u32> {
    let mut global = GlobalFile::open(open_data_options())?;

    let prev = global.read_global_count().unwrap_or(0u32);

    let new_count = count + prev;

    if count != 0 {
        global.seek_global_count()?;
        global.write_global_count(new_count)?;
    }

    Ok(new_count)
}

fn update_day_count(count : u32) -> Result<u32> {
    let date = Utc::now();

    let mut today_file = DayFile::open(&date.date(), open_data_options())?;

    let prev = today_file.read_global_count().unwrap_or(0u32);

    let new_count = prev + count;

    if count != 0 {
        today_file.seek_global_count()?;
        today_file.write_global_count(new_count)?;

        let cur_key = date.format("%H%M").to_string();

        if let Some((pre_key, pre_count)) = today_file.read_entry(EntryLoc::Last)? {
            if pre_key == cur_key {
                today_file.update_count(pre_count + count, EntryLoc::Previous)?;
            } else {
                today_file.add_entry(cur_key, count)?;
            }
        } else {
            today_file.add_entry(cur_key, count)?;
        }
    }

    Ok(new_count)
}

fn main() -> Result<()> {
    let count = keyrd_fetch()?;

    let new_global_count = update_global_count(count)?;
    let today_count = update_day_count(count)?;

    println!("{} ({} today)", new_global_count, today_count);

    Ok(())
}
