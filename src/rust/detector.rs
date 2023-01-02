mod utils;

use rppal::gpio::{Gpio, Level};
use std::{thread, sync::mpsc::Sender};

const LED_PIN: u8 = 17;
const MOTION_BCM_PIN: u8 = 22;

fn motion_thread(sendr: Sender<utils::States>) -> thread::JoinHandle<i16> {
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
              sendr.send(utils::States::RECORDING).expect("Failed to send");
          } else {
              led_output.set_low();
          };
          thread::sleep(time::Duration::from_millis(10));
      };
    })
  }
