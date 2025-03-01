use rppal::pwm::{Channel, Polarity, Pwm};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct Piezo {
    pwm: Arc<Mutex<Pwm>>,
}

impl Piezo {
    pub fn new(gpio_pin: Channel) -> Self {
        let pwm = Pwm::with_frequency(gpio_pin, 440.0, 0.0, Polarity::Normal, true)
            .expect("Failed to initialize PWM");

        Piezo {
            pwm: Arc::new(Mutex::new(pwm)),
        }
    }

    pub fn frequency(&self, value: f64) {
        let mut pwm = self.pwm.lock().unwrap();
        pwm.set_frequency(value, pwm.duty_cycle().unwrap())
            .expect("Failed to set frequency");
    }

    pub fn start(&self, frequency: Option<f64>) {
        let mut pwm = self.pwm.lock().unwrap();
        if let Some(freq) = frequency {
            pwm.set_frequency(freq, pwm.duty_cycle().unwrap())
                .expect("Failed to set frequency");
        }
        pwm.set_duty_cycle(1.0).expect("Failed to start piezo");
    }

    pub fn stop(&self) {
        let mut pwm = self.pwm.lock().unwrap();
        pwm.set_duty_cycle(0.0).expect("Failed to stop piezo");
    }

    pub fn beep(&self, frequency: f64, timeout: f64, blocking: bool) -> bool {
        if blocking {
            self.start(Some(frequency));
            thread::sleep(Duration::from_secs_f64(timeout));
            self.stop();
            true
        } else {
            let pwm_clone = Arc::clone(&self.pwm);
            thread::spawn(move || {
                let mut pwm = pwm_clone.lock().unwrap();
                pwm.set_frequency(frequency, pwm.duty_cycle().unwrap())
                    .expect("Failed to set frequency");
                pwm.set_duty_cycle(1.0).expect("Failed to start piezo");
                thread::sleep(Duration::from_secs_f64(timeout));
                pwm.set_duty_cycle(0.0).expect("Failed to stop piezo");
            });
            true
        }
    }
}
