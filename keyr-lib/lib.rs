use chrono::{Date, Utc};
use chrono::offset::TimeZone;
use std::fs::{OpenOptions, File};
use std::path::{Path, PathBuf};
use std::io::{Result, Read, Write, Seek, SeekFrom};

pub trait CounterFile {
    fn seek_global_count(&mut self) -> Result<()>;
    fn read_global_count(&mut self) -> Result<u32>;
    fn write_global_count(&mut self, count : u32) -> Result<()>;
}

impl CounterFile for File {
    fn seek_global_count(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;

        Ok(())
    }

    fn read_global_count(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;

        Ok(u32::from_le_bytes(buf))
    }

    fn write_global_count(&mut self, count : u32) -> Result<()> {
        self.write_all(&count.to_le_bytes())
    }
}

pub struct GlobalFile(File);

impl GlobalFile {
    pub fn open(opts : OpenOptions) -> Result<GlobalFile> {
        Ok(GlobalFile(opts.open(data_path("counter")?)?))
    }
}

impl CounterFile for GlobalFile {
    fn seek_global_count(&mut self) -> Result<()> {
        self.0.seek_global_count()
    }

    fn read_global_count(&mut self) -> Result<u32> {
        self.0.read_global_count()
    }

    fn write_global_count(&mut self, count : u32) -> Result<()> {
        self.0.write_global_count(count)
    }
}

pub enum EntryLoc {
    First,
    Next,
    Previous,
    Last,
}

impl EntryLoc {
    pub fn to_seek_from(&self) -> SeekFrom {
        match self {
            EntryLoc::First => SeekFrom::Start(4),
            EntryLoc::Next => SeekFrom::Current(0),
            EntryLoc::Previous => SeekFrom::Current(-8),
            EntryLoc::Last => SeekFrom::End(-8)
        }
    }
}

pub struct DayFile(File);

impl CounterFile for DayFile {
    fn seek_global_count(&mut self) -> Result<()> {
        self.0.seek_global_count()
    }

    fn read_global_count(&mut self) -> Result<u32> {
        self.0.read_global_count()
    }

    fn write_global_count(&mut self, count : u32) -> Result<()> {
        self.0.write_global_count(count)
    }
}

impl DayFile {
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

    pub fn read_entry(&mut self, entry : EntryLoc) -> Result<Option<(String, u32)>> {
        match self.0.seek(entry.to_seek_from())
            .and_then(|_| self.read_key()) {
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

    pub fn add_entry(&mut self, key : String, count : u32) -> Result<()> {
        self.0.write_all(key.as_bytes())?;
        self.0.write_all(&count.to_le_bytes())
    }

    pub fn update_count(&mut self, count : u32, entry : EntryLoc) -> Result<()> {
        self.0.seek(entry.to_seek_from())?;
        self.0.seek(SeekFrom::Current(4))?;

        self.0.write_all(&count.to_le_bytes())
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
