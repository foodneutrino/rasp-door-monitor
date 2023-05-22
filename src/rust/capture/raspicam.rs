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
            .set_format(RASPICAM_FORMAT::RASPICAM_FORMAT_RGB);

        match raspicam.open(true) {
            Ok(_) => {
                println!("****** Camera Activated");
                loop {
                    take_photo(&mut raspicam);
        
                    // take 1 picture every seconds, so sleep allowing slight processing time
                    if let Ok(sleep_time) = Duration::milliseconds(700).to_std() {
                        thread::sleep(sleep_time)
                    };
                }
            },
            Err(err) => println!("Camera Error: {:?}", err),
            
        };
        1
    })
}

fn take_photo(camera: &mut RaspiCam) {
    let image_name = format!("./door-{}.jpg", Local::now().format("%Y%m%d-%T"));
    match camera.grab() {
        Ok(raw_bytes) => {
            println!("****** Storing to {}", image_name);
            match RgbImage::from_raw(1640, 1232, raw_bytes.to_vec()) {
                Some(img) => img.save_with_format(image_name, ImageFormat::Jpeg).unwrap(),
                None => println!("Image Conversion Error"),
            };
        },
        Err(err) => println!("Capture Error: {:?}", err),
    };
}
