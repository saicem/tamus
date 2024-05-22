use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::windows::prelude::OpenOptionsExt;
use std::path::PathBuf;

use windows::Win32::Storage::FileSystem::FILE_SHARE_READ;

use crate::app::data::tmus_tick::TmusTick;

/// 2.85kB one year.
///
/// The first 8 bytes in file means the first day from `UNIX_EPOCH`.
/// After which, every 8 bytes represent the starting index of the corresponding day in record.bin.
#[derive(Debug)]
pub struct IndexBinFile {
    file: File,
    start_day: u64,
    index: Vec<u64>,
}

impl IndexBinFile {
    /// Create index.bin if not exist, and initialize it if empty.
    pub fn new(data_dir: &PathBuf) -> IndexBinFile {
        let mut file = Self::init_file(data_dir);
        let mut index = read_index(&mut file);
        let start_day = if index.is_empty() {
            let today = TmusTick::now().day();
            file.write(&today.to_le_bytes()).unwrap();
            today
        } else {
            index.pop().unwrap()
        };
        Self {
            file,
            start_day,
            index,
        }
    }

    pub fn write_index(&mut self, value: u64) {
        self.index.push(value);
        self.file.write(&value.to_le_bytes()).unwrap();
    }

    /// Query index range based on date.
    ///
    /// If the return value is u64::MAX, it indicates the end of the record file.
    pub fn query_index(&mut self, start_day: u64, end_day: u64) -> (u64, u64) {
        // the start index of given day
        let calculate_index = |day: u64| -> u64 {
            let dif = (day - self.start_day) as usize;
            if dif <= 0 {
                0
            } else if dif >= self.index.len() {
                u64::MAX
            } else {
                self.index.get(dif).unwrap().clone()
            }
        };
        let start_index = calculate_index(start_day);
        let end_index = calculate_index(end_day);
        (start_index, end_index)
    }

    fn init_file(data_dir: &PathBuf) -> File {
        OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .share_mode(FILE_SHARE_READ.0)
            .open(data_dir.join("index.bin"))
            .expect("open index.bin failed.")
    }
}

fn read_index(file: &mut File) -> Vec<u64> {
    let mut buf: [u8; 8] = [0; 8];
    let mut ret = Vec::new();
    while file.read(&mut buf).unwrap() != 0 {
        ret.push(u64::from_le_bytes(buf));
    }
    ret
}
