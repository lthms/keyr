use std::fs::OpenOptions;
use std::io::{Result, Read, Write, Seek, SeekFrom};
use std::os::unix::net::UnixStream;

fn muu_fetch() -> Result<u32> {
    let mut stream = UnixStream::connect("/tmp/mud.socket")?;
    let mut count_buff = [0u8; 4];

    stream.read_exact(&mut count_buff)?;

    Ok(u32::from_le_bytes(count_buff))
}

fn main() -> Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("muu").unwrap();
    let count = muu_fetch()?;
    let mut buff = [0u8; 4];

    let count_path = xdg_dirs.place_config_file("counter")?;

    let mut count_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(count_path)?;

    let prev_count = count_file.read_exact(&mut buff)
        .map(|_| u32::from_le_bytes(buff))
        .unwrap_or(0u32);

    let new_count = prev_count + count;

    count_file.seek(SeekFrom::Start(0))?;
    count_file.write_all(&new_count.to_le_bytes())?;

    println!("{}", new_count);


    Ok(())
}
