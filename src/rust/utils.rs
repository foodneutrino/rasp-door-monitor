use chrono::{naive::NaiveDateTime, Duration, Local};
use regex::Regex;
use std::fmt;
use std::fs::{read_dir, remove_file};
use std::path::Path;

pub enum States {
    MONITORING,
    RECORDING,
    STORING,
}

impl fmt::Display for States {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            States::MONITORING => write!(f, "Monitoring"),
            States::RECORDING => write!(f, "Recording"),
            States::STORING => write!(f, "Storing"),
        }
    }
}

pub fn get_file_pattern_cwd(pattern: &Regex) -> Vec<String> {
    read_dir(Path::new("./"))
        .unwrap()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                e.path().file_name().and_then(|n| {
                    n.to_str().map(|s| {
                        pattern
                            .captures(s)
                            .map(|group| String::from(group.name("timestamp").unwrap().as_str()))
                    })
                })
            })
        })
        .filter(|filename| filename.is_some())
        .map(|filename| filename.unwrap())
        .collect::<Vec<String>>()
}

pub fn clean_old_photos(photo_count_to_keep: i64, time_re: &Regex) {
    let oldest_timestamp = Local::now() - Duration::seconds(photo_count_to_keep);

    let old_names = get_file_pattern_cwd(time_re)
        .iter()
        .filter(|date_str| {
            NaiveDateTime::parse_from_str(date_str, "%Y%m%d-%H:%M:%S").unwrap()
                <= oldest_timestamp.naive_local()
        })
        .map(|date| String::from(date))
        .collect::<Vec<String>>();

    for dates in old_names.iter() {
        let filename = format!("door-{}.jpg", dates);
        println!("****** Deleting {}", filename);
        if let Err(e) = remove_file(filename) {
            println!("****** Error: {:?}", e);
        };
    }
}
