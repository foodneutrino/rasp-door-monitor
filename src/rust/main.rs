extern crate rascam;
extern crate chrono;
extern crate regex;
extern crate rppal;
extern crate aws_config;
extern crate aws_sdk_s3;
extern crate tokio;

pub mod utils;
mod detector;
mod capture;
mod storage;

use std::sync::mpsc::channel;
use detector::reed_detector;
use capture::camera;
use storage::s3_sync::connect;

fn main() {
    let storage_destination = connect("us-east-1").unwrap();

    let (detector_s, detector_r) = channel();

    let motion_detector = reed_detector::reed_thread(detector_s);
    let camera_t = camera::photo_thread(detector_r, Box::new(storage_destination));

    camera_t.join().unwrap();
    motion_detector.join().unwrap();
}
