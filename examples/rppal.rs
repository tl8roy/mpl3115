use std::thread;
/// This is a simple example that prints out the atlitude as fast as it can while timing it
/// You can also output the temperature or pressure
/// It uses the Raspberry Pi as the processor, but any HAL crate should work.
use std::time::{Duration, Instant};

use rppal::i2c::I2c;

use mpl3115::*;

fn main() -> ! {
    let mut i2c = I2c::new().unwrap();

    let mut mpl3115 = MPL3115A2::new(i2c, PressureAlt::Altitude).unwrap();

    mpl3115.activate();

    mpl3115.set_oversample_rate(3);

    loop {
        /*mpl3115.change_reading_type(PressureAlt::Pressure).unwrap();

        println!("Press: {}, Temp: {}",mpl3115.take_one_pa_reading().unwrap(),mpl3115.take_one_temp_reading().unwrap());

        mpl3115.change_reading_type(PressureAlt::Altitude).unwrap();*/
        let timer = Instant::now();
        let alt = mpl3115.take_one_pa_reading().unwrap();

        println!("{} {}", timer.elapsed().as_secs_f64(), alt);

        //thread::sleep(Duration::from_secs(5));
    }
}
