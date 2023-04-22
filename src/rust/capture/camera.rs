use std::{thread, sync::mpsc::Receiver};
use std::{fs::{File, remove_file}, path::PathBuf};
use std::io::Write;
use regex::Regex;
use rascam::{SimpleCamera, info};
use chrono::{Local, Duration};
use crate::utils::{States, clean_old_photos, get_file_pattern_cwd};
use crate::storage::base::StorageEngine;

pub fn photo_thread(recv: Receiver<States>, storage: Box<dyn StorageEngine>) -> thread::JoinHandle<i16> {
    let info = info().unwrap();
    if info.cameras.len() < 1 {
        println!("****** Found 0 cameras. Exiting");
        ::std::process::exit(1);
    }
    println!("****** Camera Info:\n{}", info);
    println!("------------\n");
    
    thread::spawn(move || {
      let mut state = States::MONITORING;
      let time_re: &Regex = &Regex::new(r"^door-(?P<timestamp>\d{8}-\d{2}:\d{2}:\d{2}).jpg$").unwrap();
      let onboard_camera = &info.cameras[0];
      let mut camera = SimpleCamera::new(onboard_camera.clone()).unwrap();
      let reuable_camera: &mut SimpleCamera = &mut camera;
      reuable_camera.activate().unwrap();
  
      println!("****** Camera activated");
      let mut pic_counter: i16 = 0;
      loop {
        pic_counter += 1;
        // simple_sync(&info.cameras[0]);
        take_photo(reuable_camera);
  
        if let States::MONITORING = state {
          if let Ok(s) = recv.try_recv() {
            println!("****** Existing State: {}", state);
            state = s;
            println!("****** State Change: {}", state);
          };
        };
  
        match state {
          States::MONITORING => {
            clean_old_photos(5, time_re);
            pic_counter = 0;
          },
          States::RECORDING => {
            if pic_counter >= 10 {
              state = States::STORING;
              println!("****** State Change: {}", state);
            }
          },
          States::STORING => {
            // Store the captured images and reset state
            // which includes clearing the channel buffer
            persist_all_images(&storage, time_re);
            state = States::MONITORING;
            let msgs = recv.try_iter().collect::<Vec<States>>();
            println!("****** State Change: {}; dropped {} messages", state, msgs.len());
            pic_counter = 0;
          },
        }
        
        // take 1 picture every seconds, so sleep allowing slight processing time
        if let Ok(sleep_time) = Duration::milliseconds(950).to_std() {
          thread::sleep(sleep_time)
        };
      };
    })
  }
  
  fn take_photo(camera: &mut SimpleCamera) {
      println!("****** Camera Taking picture");
      let b = camera.take_one().unwrap();
      let image_name = format!("./door-{}.jpg", Local::now().format("%Y%m%d-%T"));
      println!("****** Storing to {}", image_name);
      File::create(image_name).unwrap().write_all(&b).unwrap();
  }

fn persist_all_images(storage_type: &Box<dyn StorageEngine>, file_pattern: &Regex) {
  let files_to_copy = get_file_pattern_cwd(file_pattern);
  for filename in files_to_copy {
    if let Ok(()) = storage_type.store(
      format!("door-{}.jpg", filename).as_str(), 
      format!("detection_{}", Local::now().format("%Y%m%d_%T")).as_str()
    ) {
      let mut delete_file = PathBuf::new();
      delete_file.set_file_name(filename);
      if let Err(err) = remove_file(delete_file.as_path()) {
         println!("Remove File Failed: {}", err);
      }
    };
  };
  println!("******* Persisted");
}
