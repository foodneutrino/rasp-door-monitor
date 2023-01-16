extern crate rascam;
extern crate chrono;
extern crate regex;
extern crate rppal;

mod camera;
mod reed_detector;
pub mod utils;

use std::sync::mpsc::channel;

fn main() {
    let (detector_s, detector_r) = channel();

    let motion_detector = reed_detector::reed_thread(detector_s);
    let camera_t = camera::photo_thread(detector_r);

    camera_t.join().unwrap();
    motion_detector.join().unwrap();
}
