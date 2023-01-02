extern crate rascam;
extern crate chrono;
extern crate regex;
extern crate rppal;

mod camera
mod utils
mod detector

use std::{thread, sync::mpsc::channel};

fn main() {
    let (detector_s, detector_r) = channel();

    let motion_detector = motion_thread(detector_s);
    let camera_t = photo_thread(detector_r);

    camera_t.join().unwrap();
    motion_detector.join().unwrap();
}