use rppal::gpio::{Gpio, Level};
use std::{thread, sync::mpsc::Sender};
use chrono::Duration;
use super::utils::States;

const LED_PIN: u8 = 17;
const BCM_GPIO: u8 = 23;

pub fn reed_thread(sendr: Sender<States>) -> thread::JoinHandle<i16> {
    let gpio = Gpio::new().unwrap();
  
    println!("Initializing GPIO:\n");
    println!("------------\n");
  
    let led_pin = gpio.get(LED_PIN).unwrap();
    let motion_pin = gpio.get(BCM_GPIO).unwrap();
    
    println!("Levels | Led: {} | Motion: {}", led_pin.read(), motion_pin.read());
    let mut led_output = led_pin.into_output();
    let motion_input = motion_pin.into_input_pullup();
    thread::spawn(move || {
      loop {
          if motion_input.read() == Level::Low {
              println!("****** Motion detected");
              led_output.set_high();
              sendr.send(States::RECORDING).expect("Failed to send");
          } else {
              led_output.set_low();
          };
          if let Ok(sleep_time) = Duration::milliseconds(10).to_std() {
            thread::sleep(sleep_time)
          };
      };
    })
  }
