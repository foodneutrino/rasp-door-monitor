use chrono::{Duration, Local};
use rascam::{info, SimpleCamera};
use std::fs::File;
use std::thread;
use std::io::Write;

pub fn photo_thread() -> thread::JoinHandle<i16> {
    let info = info().unwrap();
    if info.cameras.len() < 1 {
        println!("****** Found 0 cameras. Exiting");
        ::std::process::exit(1);
    }
    println!("****** Camera Info:\n{}", info);
    println!("------------\n");

    thread::spawn(move || {
        let onboard_camera = &info.cameras[0];
        let mut camera = SimpleCamera::new(onboard_camera.clone()).unwrap();
        let reuable_camera: &mut SimpleCamera = &mut camera;
        reuable_camera.activate().unwrap();

        println!("****** Camera activated");
        loop {
            take_photo(reuable_camera);

            // take 1 picture every seconds, so sleep allowing slight processing time
            if let Ok(sleep_time) = Duration::milliseconds(950).to_std() {
                thread::sleep(sleep_time)
            };
        }
    })
}

fn take_photo(camera: &mut SimpleCamera) {
    println!("****** Camera Taking picture");
    let b = camera.take_one().unwrap();
    let image_name = format!("./door-{}.jpg", Local::now().format("%Y%m%d-%T"));
    println!("****** Storing to {}", image_name);
    File::create(image_name).unwrap().write_all(&b).unwrap();
}
