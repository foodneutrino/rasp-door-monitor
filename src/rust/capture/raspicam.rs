use chrono::{Duration, Local};
use image::{RgbImage, ImageFormat};
use raspicam_rs::{
    bindings::RASPICAM_FORMAT,
    RaspiCam,
};
use std::thread;

pub fn photo_thread() -> thread::JoinHandle<i16> {
    thread::spawn(|| {
        let mut raspicam = raspicam_rs::RaspiCam::new();
        raspicam
            .set_capture_size(1640, 1232)
            .set_frame_rate(60)
            .set_format(RASPICAM_FORMAT::RASPICAM_FORMAT_RGB)
            .open(true)
            .unwrap();

        println!("****** Camera activated");
        loop {
            take_photo(&mut raspicam);

            // take 1 picture every seconds, so sleep allowing slight processing time
            if let Ok(sleep_time) = Duration::milliseconds(950).to_std() {
                thread::sleep(sleep_time)
            };
        }
    })
}

fn take_photo(camera: &mut RaspiCam) {
    println!("****** Camera Taking picture");
    let image_name = format!("./door-{}.jpg", Local::now().format("%Y%m%d-%T"));
    let img = RgbImage::from_raw(1640, 1232, camera.grab().unwrap().to_vec()).unwrap();
    println!("****** Storing to {}", image_name);
    img.save_with_format(image_name, ImageFormat::Jpeg).unwrap();
}
