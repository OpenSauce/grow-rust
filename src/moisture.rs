use rppal::gpio::{Gpio, InputPin, Trigger};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const MOISTURE_PINS: [u8; 4] = [23, 8, 25, 4];

#[derive(Debug)]
pub struct MoistureSensor {
    gpio_pin: Arc<Mutex<InputPin>>,
    count: Arc<Mutex<u32>>,
    reading: Arc<Mutex<f64>>,
    history: Arc<Mutex<Vec<f64>>>,
    history_length: usize,
    last_pulse: Arc<Mutex<Instant>>,
    new_data: Arc<Mutex<bool>>,
    wet_point: f64,
    dry_point: f64,
    time_last_reading: Arc<Mutex<Instant>>,
}

impl MoistureSensor {
    pub fn new(channel: usize, wet_point: Option<f64>, dry_point: Option<f64>) -> Self {
        let gpio = Gpio::new().expect("Failed to initialize GPIO");
        let pin_num = MOISTURE_PINS[channel - 1];
        let pin = Arc::new(Mutex::new(
            gpio.get(pin_num).expect("Invalid GPIO pin").into_input(),
        ));

        let count = Arc::new(Mutex::new(0));
        let reading = Arc::new(Mutex::new(0.0));
        let history = Arc::new(Mutex::new(vec![]));
        let last_pulse = Arc::new(Mutex::new(Instant::now()));
        let new_data = Arc::new(Mutex::new(false));
        let time_last_reading = Arc::new(Mutex::new(Instant::now()));

        let count_clone = Arc::clone(&count);
        let reading_clone = Arc::clone(&reading);
        let history_clone = Arc::clone(&history);
        let last_pulse_clone = Arc::clone(&last_pulse);
        let new_data_clone = Arc::clone(&new_data);
        let time_last_reading_clone = Arc::clone(&time_last_reading);
        let pin_clone = Arc::clone(&pin);

        {
            let mut pin_locked = pin.lock().unwrap();
            pin_locked
                .set_interrupt(Trigger::RisingEdge, Some(Duration::from_millis(1)))
                .expect("Failed to set interrupt");
        }

        thread::spawn(move || {
            loop {
                let mut pin_locked = pin_clone.lock().unwrap();
                if pin_locked.poll_interrupt(true, None).is_ok() {
                    let mut count = count_clone.lock().unwrap();
                    *count += 1;
                    *last_pulse_clone.lock().unwrap() = Instant::now();

                    let elapsed = time_last_reading_clone
                        .lock()
                        .unwrap()
                        .elapsed()
                        .as_secs_f64();
                    if elapsed >= 1.0 {
                        let mut reading = reading_clone.lock().unwrap();
                        *reading = *count as f64 / elapsed;

                        let mut history = history_clone.lock().unwrap();
                        history.insert(0, *reading);
                        history.truncate(200);

                        *count = 0;
                        *time_last_reading_clone.lock().unwrap() = Instant::now();
                        *new_data_clone.lock().unwrap() = true;
                    }
                }
                thread::sleep(Duration::from_millis(1));
            }
        });

        MoistureSensor {
            gpio_pin: pin,
            count,
            reading,
            history,
            history_length: 200,
            last_pulse,
            new_data,
            wet_point: wet_point.unwrap_or(0.7),
            dry_point: dry_point.unwrap_or(27.6),
            time_last_reading,
        }
    }

    pub fn get_history(&self) -> Vec<f64> {
        let history = self.history.lock().unwrap();
        history
            .iter()
            .map(|&moisture| {
                let saturation = (moisture - self.dry_point) / self.get_range();
                saturation.clamp(0.0, 1.0)
            })
            .collect()
    }

    pub fn set_wet_point(&mut self, value: Option<f64>) {
        self.wet_point = value.unwrap_or(*self.reading.lock().unwrap());
    }

    pub fn set_dry_point(&mut self, value: Option<f64>) {
        self.dry_point = value.unwrap_or(*self.reading.lock().unwrap());
    }

    pub fn get_moisture(&self) -> f64 {
        let mut new_data = self.new_data.lock().unwrap();
        *new_data = false;
        *self.reading.lock().unwrap()
    }

    pub fn is_active(&self) -> bool {
        self.last_pulse.lock().unwrap().elapsed() < Duration::from_secs(1)
            && *self.reading.lock().unwrap() > 0.0
            && *self.reading.lock().unwrap() < 28.0
    }

    pub fn has_new_data(&self) -> bool {
        *self.new_data.lock().unwrap()
    }

    pub fn get_range(&self) -> f64 {
        self.wet_point - self.dry_point
    }

    pub fn get_saturation(&self) -> f64 {
        let moisture = self.get_moisture();
        let saturation = (moisture - self.dry_point) / self.get_range();
        saturation.clamp(0.0, 1.0)
    }
}
