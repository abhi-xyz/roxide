use std::error::Error;
use std::fs::{self, create_dir_all, rename, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;
use std::vec;

use dirs::data_dir;
use log::debug;

//
//

/// # LogId unique id which represents year, month, date, hour, minute and second
/// in this order itself. ("%Y%m%d%H%M%S")
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct LogId {
    pub num: u64,
}

impl From<u64> for LogId {
    fn from(value: u64) -> Self {
        LogId { num: value }
    }
}

impl FromStr for LogId {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().parse() {
            Ok(id) => Ok(LogId { num: id }),
            Err(e) => Err(e),
        }
    }
}

// current issue
// BUG: when i remove the dir from some/dir to trash it wont work
//
// Possibilities
//
// HACK: no need to test mutiple files/dirs Possibilities
//       since it will become single file/dir in for loop
//
// 1. single file from root [  NOTE: tested ]
//
// 2. mutiple files from root by specifiying name
// 3. mutiple files from root using glob wildcard
// 4. single dir from root [  NOTE: tested ]
//
// 5. mutiple dirs from root
// 6. mutiple dirs from root using glob wildcard
//
// 1. single file from some/dir [  NOTE: tested ]
//
// 2. mutiple files from some/dir by specifiying name
// 3. mutiple files from some/dir using glob wildcard
// 4. single dir from some/dir [  NOTE: tested ]
// 5. mutiple dirs from some/dir
// 6. mutiple dirs from some/dir using glob wildcard

//

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TrashHistory {
    pub original_path: String,
    pub trash_path: String,
    pub deleted_at: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TrashMeta {
    pub unique_id: String,
    pub history: TrashHistory,
}

pub fn write_history(
    unique_id: String,
    original_path: String,
    trash_path: String,
) -> anyhow::Result<()> {
    let log_dir = data_dir().unwrap().join("roxide");
    if !log_dir.exists() {
        create_dir_all(log_dir).unwrap();
    }
    let log_file = data_dir().unwrap().join("roxide/roxide_history.log");
    if !log_file.exists() {
        File::create(&log_file).unwrap();
    }
    let mut file = OpenOptions::new()
        .create(true) // Create the file if it doesn't exist
        .append(true) // Append to the file if it already exists
        .open(log_file)?;

    writeln!(file, "{}", unique_id)?;
    writeln!(file, "{}", original_path)?;
    writeln!(file, "{}", trash_path)?;
    writeln!(file, "----------------------------")?;

    Ok(())
}

pub fn read_history() -> Result<(), Box<dyn Error>> {
    let log_file = data_dir().unwrap().join("roxide/roxide_history.log");
    let file = fs::File::open(&log_file)?;
    let reader = BufReader::new(file);

    let mut a_vec: Vec<String> = vec::Vec::new();
    for line in reader.lines() {
        let line = line?;
        a_vec.push(line);
    }
    revert(
        a_vec.iter().nth_back(1).unwrap().to_string(),
        a_vec.iter().nth_back(2).unwrap().to_string(),
    )?;

    let vec_lng = a_vec.len();
    debug!("vec len: {}", &vec_lng);

    if vec_lng > 160 {
        a_vec.truncate(160);
    }
    let mut new_file = fs::File::create(log_file)?;
    for i in a_vec {
        writeln!(new_file, "{}", i)?;
    }

    Ok(())
}

fn revert(from: String, to: String) -> Result<(), Box<dyn Error>> {
    if !Path::new(&from).exists() {
        println!("File Doesn't Exist in Trash dir");
    } else {
        rename(from, to)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn get_last_log() {
        #[allow(clippy::useless_vec)]
        let dummy_log = vec![
            "-------------",
            "-------------",
            "-------------",
            "-------------",
            "-------------",
            "20241112214434",
            "/home/abhi/projects/abhi/github/roxide/file004.org",
            "/home/abhi/.local/share/Trash/files/file004.2024-11-12_21:44:34.org",
            "20241112214434",
            "----------------------------",
        ];
        assert_eq!(
            "20241112214434",
            &dummy_log.iter().nth_back(4).unwrap().to_string()
        );
        assert_eq!(
            "/home/abhi/projects/abhi/github/roxide/file004.org",
            &dummy_log.iter().nth_back(3).unwrap().to_string()
        );
        assert_eq!(
            "/home/abhi/.local/share/Trash/files/file004.2024-11-12_21:44:34.org",
            &dummy_log.iter().nth_back(2).unwrap().to_string()
        );
        assert_eq!(
            "20241112214434",
            &dummy_log.iter().nth_back(1).unwrap().to_string()
        );
        assert_eq!(
            "----------------------------",
            &dummy_log.iter().nth_back(0).unwrap().to_string()
        );
    }
}