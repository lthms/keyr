use std::fs::File;
use std::path::PathBuf;
use std::io::{Result, Read, Write, Seek, SeekFrom};

pub fn data_path(name : &str) -> Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("muu").unwrap();

    xdg_dirs.place_config_file(name)
}

pub fn read_count(file : &mut File) -> Result<u32> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;

    Ok(u32::from_le_bytes(buf))
}

pub fn write_count(file : &mut File, count : u32) -> Result<()> {
    file.write_all(&count.to_le_bytes())?;

    Ok(())
}

pub fn read_key(file : &mut File) -> Result<String> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;

    // FIXME
    Ok(String::from_utf8(Vec::from(buf.as_ref())).unwrap())
}

pub fn write_key(file : &mut File, key : &str) -> Result<()> {
    file.write_all(key.as_bytes())?;

    Ok(())
}

pub fn read_last_entry(file : &mut File) -> Result<Option<(String, u32)>> {
    match file.seek(SeekFrom::End(-8)) {
        Ok(_) => {
            let key = read_key(file)?;
            let count = read_count(file)?;

            Ok(Some((key, count)))
        },
        _ => {
            Ok(None)
        }
    }
}
