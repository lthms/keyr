use chrono::Utc;
use std::fs::{File, OpenOptions};
use std::io::{Result, Read, Write, Seek, SeekFrom};
use std::os::unix::net::UnixStream;

fn muu_fetch() -> Result<u32> {
    let mut stream = UnixStream::connect("/tmp/mud.socket")?;
    let mut count_buff = [0u8; 4];

    stream.read_exact(&mut count_buff)?;

    Ok(u32::from_le_bytes(count_buff))
}

fn open_data_file(name : &str) -> Result<File> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("muu").unwrap();

    let path = xdg_dirs.place_config_file(name)?;

    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
}

fn update_global_count(count : u32) -> Result<u32> {
    let mut buff = [0u8; 4];
    let mut global_file = open_data_file("counter")?;

    let prev = global_file.read_exact(&mut buff)
        .map(|_| u32::from_le_bytes(buff))
        .unwrap_or(0u32);

    let new_count = count + prev;

    global_file.seek(SeekFrom::Start(0))?;
    global_file.write_all(&new_count.to_le_bytes())?;

    Ok(new_count)
}

fn update_hourly_count(count : u32) -> Result<u32> {
    let date = Utc::now();
    let mut key_buff = [0u8; 2];
    let mut count_buff = [0u8; 4];

    let mut today_file = open_data_file(&date.format("%Y%m%d").to_string())?;

    let cur_key = date.format("%H").to_string();

    // read the key
    match today_file.seek(SeekFrom::End(-6)) {
        Ok(_) => { // there is something to read
            today_file.read_exact(&mut key_buff)?;
            let pre_key = String::from_utf8(Vec::from(key_buff.as_ref())).unwrap();

            today_file.read_exact(&mut count_buff)?;
            let pre_count = u32::from_le_bytes(count_buff);

            if pre_key == cur_key {
                let new_count = pre_count + count;

                today_file.seek(SeekFrom::End(-4))?;
                today_file.write_all(&new_count.to_le_bytes())?;

                Ok(new_count)
            } else {
                today_file.write_all(cur_key.as_bytes())?;
                today_file.write_all(&count.to_le_bytes())?;
                Ok(count)
            }
        },
        _ => { // nothing to read yet
            today_file.seek(SeekFrom::Start(0))?;
            today_file.write_all(cur_key.as_bytes())?;
            today_file.write_all(&count.to_le_bytes())?;

            Ok(count)
        }
    }
}

fn main() -> Result<()> {
    let count = muu_fetch()?;

    let new_global_count = update_global_count(count)?;
    let new_hourly_count = update_hourly_count(count)?;

    println!("{} ({} this hour)", new_global_count, new_hourly_count);

    Ok(())
}
