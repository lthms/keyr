use chrono::Utc;
use std::io::{Result, Read, Write, Seek, SeekFrom};
use std::os::unix::net::UnixStream;
use std::fs::{OpenOptions, File};
use std::path::Path;

fn open_data_file(path : &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
}

fn muu_fetch() -> Result<u32> {
    let mut stream = UnixStream::connect("/tmp/mud.socket")?;
    let mut count_buff = [0u8; 4];

    stream.read_exact(&mut count_buff)?;

    Ok(u32::from_le_bytes(count_buff))
}

fn update_global_count(count : u32) -> Result<u32> {
    let mut global_file = open_data_file(&mul::data_path("counter")?)?;

    let prev = mul::read_count(&mut global_file)
        .unwrap_or(0u32);

    let new_count = count + prev;

    global_file.seek(SeekFrom::Start(0))?;
    mul::write_count(&mut global_file, new_count)?;

    Ok(new_count)
}

fn update_hourly_count(count : u32) -> Result<u32> {
    let date = Utc::now();

    let mut today_file = open_data_file(
        &mul::data_path(&date.format("%Y%m%d").to_string())?
    )?;

    let cur_key = date.format("%H%M").to_string();

    let global_count = mul::read_count(&mut today_file)
        .map(|x| x + count)
        .unwrap_or(count);

    today_file.seek(SeekFrom::Start(0))?;

    mul::write_count(&mut today_file, global_count)?;

    today_file.write_all(&global_count.to_le_bytes())?;

    if let Some((pre_key, pre_count)) =
        mul::read_last_entry(&mut today_file)? {
        if pre_key == cur_key {
            let new_count = pre_count + count;

            today_file.seek(SeekFrom::End(-4))?;
            mul::write_count(&mut today_file, new_count)?;
        } else {
            mul::write_key(&mut today_file, &cur_key)?;
            mul::write_count(&mut today_file, count)?;
        }
    } else {
        today_file.seek(SeekFrom::Start(4))?;
        mul::write_key(&mut today_file, &cur_key)?;
        mul::write_count(&mut today_file, count)?;

    }

    Ok(global_count)
}

fn main() -> Result<()> {
    let count = muu_fetch()?;

    let new_global_count = update_global_count(count)?;
    let today_count = update_hourly_count(count)?;

    println!("{} ({} today)", new_global_count, today_count);

    Ok(())
}
