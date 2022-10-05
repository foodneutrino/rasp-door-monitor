extern crate rascam;

use rascam::{CameraInfo, SimpleCamera, info};
use std::fs::File;
use std::io::Write;
use std::{thread, time};


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
        for i in 1..4 {
            println!("taking picture {}", i);
            simple_sync(&info.cameras[0], i);
        }
    });

    camera_t.join().unwrap();
}

fn simple_sync(info: &CameraInfo, pic_number: u32) {
    let mut camera = SimpleCamera::new(info.clone()).unwrap();
    camera.activate().unwrap();

    println!("Camera activating");
    let sleep_duration = time::Duration::from_millis(2000);
    thread::sleep(sleep_duration);

    println!("Camera Taking picture");
    let b = camera.take_one().unwrap();
    let image_name = format!("door{}.jpg", pic_number);
    File::create(image_name).unwrap().write_all(&b).unwrap();

    println!("Saved image as image.jpg");
}