mod utils;

use std::{thread, sync::mpsc::Receiver};
use std::{fs::{File, create_dir, copy}, path::PathBuf};
use std::io::Write;
use regex::Regex;
use rascam::{SimpleCamera, info};
use chrono::{Local, Duration};

fn photo_thread(recv: Receiver<utils::States>) -> thread::JoinHandle<i16> {
    let info = info().unwrap();
    if info.cameras.len() < 1 {
        println!("****** Found 0 cameras. Exiting");
        ::std::process::exit(1);
    }
    println!("****** Camera Info:\n{}", info);
    println!("------------\n");
    
    thread::spawn(move || {
      let mut state = utils::States::MONITORING;
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
  
        if let utils::States::MONITORING = state {
          if let Ok(s) = recv.try_recv() {
            println!("****** Existing State: {}", state);
            state = s;
            println!("****** State Change: {}", state);
          };
        };
  
        match state {
          utils::States::MONITORING => {
            utils::clean_old_photos(5, time_re);
            pic_counter = 0;
          },
          utils::States::RECORDING => {
            if pic_counter >= 10 {
              state = utils::States::STORING;
              println!("****** State Change: {}", state);
            }
          },
          utils::States::STORING => {
            // Store the captured images and reset state
            // which includes clearing the channel buffer
            persist_all_images(time_re);
            state = utils::States::MONITORING;
            let msgs = recv.try_iter().collect::<Vec<utils::States>>();
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

fn persist_all_images(file_pattern: &Regex) {
  let mut dest_path = PathBuf::new();
  dest_path.set_file_name(
    format!("./detection_{}", Local::now().format("%Y%m%d_%T"))
  );
  if let Err(e) = create_dir(dest_path.as_path()) {
    println!("****** Failed creating directory {:?}", e)
  };

  let files_to_copy = utils::get_file_pattern_cwd(file_pattern);
  for filename in files_to_copy {
    let mut source_file = PathBuf::new();
    source_file.set_file_name(
      format!("./door-{}.jpg", filename)
    );
    dest_path.push(source_file.as_path());
    if let Err(e) = copy(source_file.as_path(), dest_path.as_path()) {
      println!(
        "****** Source {} | Destination {} | Error {:?}", 
        source_file.to_str().unwrap(), 
        dest_path.to_str().unwrap(), e
      );
    } else {
      if let Err(e) = remove_file(&source_file) {
        println!("****** Source {} | Error {:?}", source_file.to_str().unwrap(), e);
      };
    };
    dest_path.pop();
  };
  println!("******* Persisted");
}
