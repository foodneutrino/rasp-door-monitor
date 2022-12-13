extern crate rascam;
extern crate chrono;
extern crate regex;

use rascam::{SimpleCamera, info};
use std::fs::File;
use std::io::Write;
use chrono::{Local, Duration, naive::NaiveDateTime};
use std::{thread, time, fs, path::Path};
use regex::Regex;

fn main() {
    let info = info().unwrap();
    if info.cameras.len() < 1 {
        println!("Found 0 cameras. Exiting");
        // note that this doesn't run destructors
        ::std::process::exit(1);
    }
    println!("Camera Info:\n{}", info);
    println!("------------\n");

    let camera_t = thread::spawn(move || {
        loop {
            let time_re: Regex = Regex::new(r"^door-(?P<timestamp>\d{8}-\d{2}:\d{2}:\d{2}).jpg$").unwrap();
            println!("Camera activating");
            let onboard_camera = &info.cameras[0];
            let mut camera = SimpleCamera::new(onboard_camera.clone()).unwrap();
            camera.activate().unwrap();

            // simple_sync(&info.cameras[0]);
            take_photo(camera);

            clean_old_photos(
                5, time_re
            );
            
            // take 1 picture every seconds
            println!("sleep");
            let sleep_duration = time::Duration::from_millis(2000);
            thread::sleep(sleep_duration);
        }
    });

    camera_t.join().unwrap();
}

// fn simple_sync(info: &CameraInfo) {
fn take_photo(mut camera: SimpleCamera) {
    println!("Camera Taking picture");
    let b = camera.take_one().unwrap();
    let image_name = format!("./door-{}.jpg", Local::now().format("%Y%m%d-%T"));
    println!("Storing to {}", image_name);
    File::create(image_name).unwrap().write_all(&b).unwrap();

    println!("Saved image as image.jpg");
}

fn clean_old_photos(photo_count_to_keep: i64, time_re: Regex) {
    let file_glob = Path::new("./");
    let oldest_timestamp = Local::now() - Duration::seconds(photo_count_to_keep);
    let paths = fs::read_dir(&file_glob).unwrap();

    let names = paths.filter_map(|entry| {
        entry.ok().and_then(|e|
          e.path().file_name()
          .and_then(|n| n.to_str().map(|s| String::from(s)))
        )
      })
      .filter_map(|old_file| {
        let image_file = &old_file.as_str()[..];
        time_re
         .captures(image_file)
         .map(|group| {String::from(group.name("timestamp").unwrap().as_str())})
      })
      .filter(|date_str| {
        NaiveDateTime::parse_from_str(date_str, "%Y%m%d-%H:%M:%S").unwrap() >= oldest_timestamp.naive_local()
      })
      .collect::<Vec<String>>();

      println!("Keeping: {} | {:?}", photo_count_to_keep, names);
}