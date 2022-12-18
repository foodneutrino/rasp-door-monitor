extern crate rascam;
extern crate chrono;
extern crate regex;
extern crate rppal;

use rascam::{SimpleCamera, info};
use std::fs::{File, remove_file, read_dir, create_dir, copy};
use std::io::Write;
use std::{time, path::{Path, PathBuf}, fmt};
use std::{thread, sync::mpsc::{channel, Receiver, Sender}};
use chrono::{Local, Duration, naive::NaiveDateTime};
use regex::Regex;
use rppal::gpio::{Gpio, Level};

enum States {
  MONITORING,
  RECORDING,
  STORING,
}
impl fmt::Display for States {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
     match *self {
         States::MONITORING => write!(f, "Monitoring"),
         States::RECORDING => write!(f, "Recording"),
         States::STORING => write!(f, "Storing"),
     }
  }
}

const LED_PIN: u8 = 17;
const MOTION_BCM_PIN: u8 = 22;

fn main() {
    let (detector_s, detector_r) = channel();

    let motion_detector = motion_thread(detector_s);
    let camera_t = photo_thread(detector_r);

    camera_t.join().unwrap();
    motion_detector.join().unwrap();
}

fn motion_thread(sendr: Sender<States>) -> thread::JoinHandle<i16> {
  let gpio = Gpio::new().unwrap();

  println!("Initializing GPIO:\n");
  println!("------------\n");

  let led_pin = gpio.get(LED_PIN).unwrap();
  let motion_pin = gpio.get(MOTION_BCM_PIN).unwrap();
  
  println!("Levels | Led: {} | Motion: {}", led_pin.read(), motion_pin.read());
  let mut led_output = led_pin.into_output();
  let motion_input = motion_pin.into_input();
  thread::spawn(move || {
    loop {
        if motion_input.read() == Level::High {
            println!("****** Motion detected");
            led_output.set_high();
            sendr.send(States::RECORDING).expect("Failed to send");
        } else {
            led_output.set_low();
        };
        thread::sleep(time::Duration::from_millis(10));
    };
  })
}

fn photo_thread(recv: Receiver<States>) -> thread::JoinHandle<i16> {
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
          persist_all_images(time_re);
          state = States::MONITORING;
          let msgs = recv.try_iter().collect::<Vec<States>>();
          println!("****** State Change: {}; dropped {} messages", state, msgs.len());
          pic_counter = 0;
        },
      }
      
      // take 1 picture every seconds, so sleep allowing slight processing time
      thread::sleep(time::Duration::from_millis(950));
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

fn get_file_pattern_cwd(pattern: &Regex) -> Vec<String> {
  read_dir(Path::new("./")).unwrap()
    .filter_map(|entry| {
      entry.ok()
        .and_then(|e| e.path().file_name()
          .and_then(|n| n.to_str()
            .map(|s| {
              pattern
                .captures(s)
                .map(|group| {
                  String::from(group.name("timestamp").unwrap().as_str())
                })
            })
          )
        )
    })
    .filter(|filename| filename.is_some())
    .map(|filename| filename.unwrap())
    .collect::<Vec<String>>()
}

fn clean_old_photos(photo_count_to_keep: i64, time_re: &Regex) {
    let oldest_timestamp = Local::now() - Duration::seconds(photo_count_to_keep);
    
    let old_names = get_file_pattern_cwd(time_re).iter()
      .filter(|date_str| {
        NaiveDateTime::parse_from_str(date_str, "%Y%m%d-%H:%M:%S").unwrap() <= oldest_timestamp.naive_local()
      })
      .map(|date| String::from(date))
      .collect::<Vec<String>>();

      for dates in old_names.iter() {
        let filename = format!("door-{}.jpg", dates);
        println!("****** Deleting {}", filename);
        if let Err(e) = remove_file(filename) {
          println!("****** Error: {:?}", e);
        };
      }
}

fn persist_all_images(file_pattern: &Regex) {
  let mut dest_path = PathBuf::new();
  dest_path.set_file_name(
    format!("./detection_{}", Local::now().format("%Y%m%d_%T"))
  );
  if let Err(e) = create_dir(dest_path.as_path()) {
    println!("****** Failed creating directory {:?}", e)
  };

  let files_to_copy = get_file_pattern_cwd(file_pattern);
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