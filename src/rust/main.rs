extern crate aws_config;
extern crate aws_sdk_s3;
extern crate chrono;
extern crate rascam;
extern crate regex;
extern crate rppal;
extern crate tokio;

mod capture;
mod detector;
mod storage;
pub mod utils;

use capture::camera;
use detector::reed_detector;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use storage::local_to_s3::connect;
use storage::base::StorageEngine;
use utils::{clean_old_photos, get_file_pattern_cwd, States};

use regex::Regex;
use std::fs::remove_file;
use std::path::PathBuf;
use chrono::Duration;
use chrono::Local;

fn main() {
    let storage_destination = connect("us-east-1").unwrap();

    let (detector_s, detector_r) = channel();
    let motion_detector = reed_detector::reed_thread(detector_s);

    let camera_t = camera::photo_thread();

    controller(camera_t, motion_detector, detector_r, Box::new(storage_destination));
}

fn controller(
    camera: thread::JoinHandle<i16>,
    detector: thread::JoinHandle<i16>,
    recv_channel: Receiver<States>,
    storage: Box<dyn StorageEngine>,
) {
    let mut state = States::MONITORING;
    let time_re: &Regex =
        &Regex::new(r"^door-(?P<timestamp>\d{8}-\d{2}:\d{2}:\d{2}).jpg$").unwrap();

    camera.join().unwrap();
    detector.join().unwrap();

    loop {
        // assume camera thread is taking 1 picture a second
        // will look for messages from detector

        match state {
            // while monitoring, either change state to record and kept past photos
            // or clean up older photos
            States::MONITORING => match recv_channel.try_recv() {
                Ok(s) => {
                    println!("State: {} => {}", state, s);
                    state = s;
                }
                Err(_) => clean_old_photos(5, time_re),
            },
            States::RECORDING => {
                // allow camera to store 10 seconds of images
                if let Ok(sleep_time) = Duration::seconds(10).to_std() {
                    thread::sleep(sleep_time)
                };
                println!("State: {} => Storing", state);
                state = States::STORING;
            }
            States::STORING => {
                // Store the captured images and reset state
                // which includes clearing the channel buffer
                persist_all_images(&storage, time_re);
                state = States::MONITORING;

                // clear any jitter from from the detector
                let msgs = recv_channel.try_iter().collect::<Vec<States>>();
                println!(
                    "****** State Change: {}; dropped {} messages",
                    state,
                    msgs.len()
                );
            }
        }
    }
}

fn persist_all_images(storage_type: &Box<dyn StorageEngine>, file_pattern: &Regex) {
    let files_to_copy = get_file_pattern_cwd(file_pattern);
    let time_base = Local::now().format("%Y%m%d_%T");
    let mut image_number: u8 = 1;
    for filename in files_to_copy {
        let file_to_store = format!("door-{}.jpg", filename);
        if let Ok(()) = storage_type.store(
            file_to_store.as_str(),
            format!("{}/detection_{:0>2}", time_base, image_number).as_str(),
        ) {
            let mut delete_file = PathBuf::new();
            delete_file.set_file_name(file_to_store);
            println!("Deleting: {}", delete_file.display());
            if let Err(err) = remove_file(delete_file.as_path()) {
                println!("Remove File Failed: {}", err);
            }
        };
        image_number += 1;
    }
    println!("******* Persisted");
}
