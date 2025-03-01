use grow_rust::{MoistureSensor, Piezo};
use rppal::pwm::Channel;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Testing Moisture Sensor...");
    let sensor = MoistureSensor::new(1, None, None);

    for _ in 0..5 {
        let moisture = sensor.get_moisture();
        let saturation = sensor.get_saturation();
        let active = sensor.is_active();
        let new_data = sensor.has_new_data();

        println!(
            "Moisture: {:.2}, Saturation: {:.2}, Active: {}, New Data: {}",
            moisture, saturation, active, new_data
        );

        thread::sleep(Duration::from_secs(1));
    }

    println!("Testing Piezo Buzzer...");
    let piezo = Piezo::new(Channel::Pwm1);
    piezo.beep(1000.0, 0.5, true);
    thread::sleep(Duration::from_secs(1));
    piezo.beep(500.0, 1.0, false);
    thread::sleep(Duration::from_secs(2));

    println!("Test complete.");
}
