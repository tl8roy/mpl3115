//! This is an embedded-hal device driver for the MPL3115A2 Altitude sensor.
//!
//! It is accurate to around 0.3m.
//!
//! There is also a temperature and pressure readings avaliable.
//!
//! See the examples folder for a blocking implementation.
//!
//! The pressure mode is likely wildly inaccurate and needs to be tested against a known good pressure sensor.

#![no_std]

mod reg;
use reg::*;

use core::fmt::Debug;

use cast::f32;

use embedded_hal::i2c::I2c;

#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2c(E),
    /// Failed to parse sensor data (Not yet used)
    InvalidData,
    /// Chip ID doesn't match expected value
    UnsupportedChip,
}

/// Device Mode
///
/// Useful for "non" blocking measurements
#[derive(Copy, Clone, PartialEq)]
pub enum Mode {
    Inactive,
    Active,
    TakingReading,
}

/// Pressure or Altitude Mode
///
/// Toggle as required
#[derive(Copy, Clone, PartialEq)]
pub enum PressureAlt {
    Pressure,
    Altitude,
}

/// `MPL3115A2` driver
///
/// Will start off deactivated and in the PressureAlt mode set
pub struct MPL3115A2<T> {
    /// The concrete I²C device implementation
    i2c: T,

    /// Mode (Inactive, Active, Taking Sample)
    mode: Mode,

    /// Pressure or Altitude Mode
    pa: PressureAlt,
}

/// Interrupt setting and status
/*pub struct Int<'a, REG, I2C> {
    dev: &'a mut MMA8452q<I2C>,
    reg: PhantomData<REG>,
}*/

impl<I2C, E> MPL3115A2<I2C>
where
    I2C: I2c<Error = E>,
    E: Debug,
{
    /// Create a new `MPL3115A2` driver from the given `I2C` peripheral
    pub fn new(i2c: I2C, pa: PressureAlt) -> Result<Self, Error<E>> {
        //Create the device
        let mut dev = Self {
            i2c,
            mode: Mode::Inactive,
            pa,
        };

        // Ensure we have the correct device ID
        if dev.get_device_id()? != DEVICE_ID {
            return Err(Error::UnsupportedChip);
        }

        //Enables the pressure and temp measurement event flags so that we can
        //test against them. This is recommended in datasheet during setup.
        //Enable all three pressure and temp event flags
        dev.write_reg(Register::PT_DATA_CFG, EVENT_FLAGS)
            .map_err(Error::I2c)?;

        //Set the required PA mode
        dev.change_reading_type(pa)?;

        Ok(dev)
    }

    /// Destroy driver instance, return `I2C` bus instance
    pub fn destroy(self) -> I2C {
        self.i2c
    }

    /// Get the `WHO_AM_I` register
    pub fn get_device_id(&mut self) -> Result<u8, Error<E>> {
        self.read_reg(Register::WHO_AM_I).map_err(Error::I2c)
    }

    /// Activate the Device
    pub fn activate(&mut self) -> Result<(), Error<E>> {
        self.reg_set_bits(Register::CTRL_REG1, DEVICE_EN)
            .map_err(Error::I2c)?;
        self.mode = Mode::Active;
        Ok(())
    }

    /// De-activate the Device
    pub fn deactivate(&mut self) -> Result<(), Error<E>> {
        self.reg_reset_bits(Register::CTRL_REG1, DEVICE_EN)
            .map_err(Error::I2c)?;
        self.mode = Mode::Inactive;
        Ok(())
    }

    /// Change between altitude and pressure
    pub fn change_reading_type(&mut self, pa: PressureAlt) -> Result<(), Error<E>> {
        match pa {
            PressureAlt::Altitude => {
                self.reg_set_bits(Register::CTRL_REG1, ALT_EN)
                    .map_err(Error::I2c)?;
            }
            PressureAlt::Pressure => {
                self.reg_reset_bits(Register::CTRL_REG1, ALT_EN)
                    .map_err(Error::I2c)?;
            }
        }

        self.pa = pa;
        Ok(())
    }

    /// Get one (blocking) Pressure or Altitude value
    pub fn take_one_pa_reading(&mut self) -> Result<f32, Error<E>> {
        //Trigger a one shot reading
        self.start_reading()?;

        //Wait for PDR bit, indicates we have new pressure data
        while !self.check_pa_reading()? {}

        //Get the data
        self.get_pa_reading()
    }

    /// Get one (blocking) Temperature value
    pub fn take_one_temp_reading(&mut self) -> Result<f32, Error<E>> {
        //Trigger a one shot reading
        self.start_reading()?;

        //Wait for TDR bit, indicates we have new temperature data
        while !self.check_temp_reading()? {}

        //Get the data
        self.get_temp_reading()
    }

    /// Clear then set the OST bit which causes the sensor to immediately take another reading
    /// Needed to sample faster than 1Hz
    pub fn start_reading(&mut self) -> Result<(), Error<E>> {
        self.reg_reset_bits(Register::CTRL_REG1, ONE_SHOT)
            .map_err(Error::I2c)?;
        self.reg_set_bits(Register::CTRL_REG1, ONE_SHOT)
            .map_err(Error::I2c)?;

        //We are now waiting for data
        self.mode = Mode::TakingReading;
        Ok(())
    }

    /// Check the PDR bit for new data
    pub fn check_pa_reading(&mut self) -> Result<bool, Error<E>> {
        let status_reg = self.read_reg(Register::STATUS).map_err(Error::I2c)?;
        Ok(status_reg & PDR != 0)
    }

    /// Check the TDR bit for new data
    pub fn check_temp_reading(&mut self) -> Result<bool, Error<E>> {
        let status_reg = self.read_reg(Register::STATUS).map_err(Error::I2c)?;

        Ok(status_reg & TDR != 0)
    }

    /// Get and process the pressure or altitude data
    pub fn get_pa_reading(&mut self) -> Result<f32, Error<E>> {
        // Read pressure registers
        let mut buf = [0u8; 3];
        self.read_regs(Register::OUT_P_MSB, &mut buf)
            .map_err(Error::I2c)?;

        //Change the device back to active
        self.mode = Mode::Active;

        // The least significant bytes l_altitude and l_temp are 4-bit,
        // fractional values, so you must cast the calulation in (float),
        // shift the value over 4 spots to the right and divide by 16 (since
        // there are 16 values in 4-bits).
        match self.pa {
            PressureAlt::Altitude => {
                let lsb = buf[2] >> 4;
                let tempcsb = f32(lsb) / 16.0;
                let int_buf = [buf[0], buf[1]];

                let altitude = f32(i16::from_be_bytes(int_buf)) + tempcsb;

                Ok(altitude)
            }
            PressureAlt::Pressure => {
                //Reads the current pressure in Pa
                // Pressure comes back as a left shifted 20 bit number
                let int_buf = [0u8, buf[0], buf[1], buf[2]];
                let mut pressure_whole: u32 = u32::from_be_bytes(int_buf);
                pressure_whole >>= 6; //Pressure is an 18 bit number with 2 bits of decimal. Get rid of decimal portion.

                buf[2] &= 0b0011_0000; //Bits 5/4 represent the fractional component
                buf[2] >>= 4; //Get it right aligned
                let pressure_decimal = f32(buf[2]) / 4.0; //Turn it into fraction

                let pressure = f32(pressure_whole) + pressure_decimal;

                Ok(pressure)
            }
        }
    }

    ///Get and process the temperature data
    pub fn get_temp_reading(&mut self) -> Result<f32, Error<E>> {
        // Read temperature registers
        let mut buf = [0u8; 2];
        self.read_regs(Register::OUT_T_MSB, &mut buf)
            .map_err(Error::I2c)?;

        //Change the device back to active
        self.mode = Mode::Active;

        //Negative temperature fix by D.D.G.
        //let mut foo_r: u16 = 0;
        let mut neg_sign = false;

        //Check for 2s compliment
        if buf[0] > 0x7F {
            let mut foo_r = u16::from_be_bytes(buf);
            foo_r = !foo_r + 1; //2’s complement
            buf = foo_r.to_be_bytes();
            buf[1] &= 0xF0;
            neg_sign = true;
        }

        // The least significant bytes l_altitude and l_temp are 4-bit,
        // fractional values, so you must cast the calulation in (float),
        // shift the value over 4 spots to the right and divide by 16 (since
        // there are 16 values in 4-bits).
        let templsb = f32(buf[1] >> 4) / 16.0; //temp, fraction of a degree

        let mut temperature = f32(buf[0]) + templsb;

        if neg_sign {
            temperature = 0.0 - temperature;
        }

        Ok(temperature)
    }

    /// Set the number of samples the device makes before saving the data
    /// Call with a rate from 0 to 7. Datasheet calls for 128 but you can set it from 1 to 128 samples.
    /// The higher the oversample rate the greater the time between data samples.
    ///
    /// Example Times:
    /// * 0 = 8ms
    /// * 3 = 30ms
    /// * 7 = 380ms
    pub fn set_oversample_rate(&mut self, mut sample_rate: u8) -> Result<(), Error<E>> {
        if sample_rate > 7 {
            sample_rate = 7; //OS cannot be larger than 0b.0111
        }
        sample_rate <<= 3; //Align it for the CTRL_REG1 register

        let mut temp_setting = self.read_reg(Register::CTRL_REG1).map_err(Error::I2c)?; //Read current settings
        temp_setting &= 0b1100_0111; //Clear out old OS bits
        temp_setting |= sample_rate; //Mask in new OS bits
        self.write_reg(Register::CTRL_REG1, temp_setting)
            .map_err(Error::I2c)
    }

    #[inline]
    fn read_reg(&mut self, reg: Register) -> Result<u8, E> {
        let mut buf = [0u8];
        self.i2c.write_read(I2C_SAD, &[reg.addr()], &mut buf)?;
        Ok(buf[0])
    }

    #[inline]
    fn read_regs(&mut self, reg: Register, buffer: &mut [u8]) -> Result<(), E> {
        self.i2c.write_read(I2C_SAD, &[reg.addr()], buffer)
    }

    #[inline]
    fn write_reg(&mut self, reg: Register, val: u8) -> Result<(), E> {
        self.i2c.write(I2C_SAD, &[reg.addr(), val])
    }

    #[inline]
    fn modify_reg<F>(&mut self, reg: Register, f: F) -> Result<(), E>
    where
        F: FnOnce(u8) -> u8,
    {
        let r = self.read_reg(reg)?;
        self.write_reg(reg, f(r))?;
        Ok(())
    }

    #[inline]
    fn reg_set_bits(&mut self, reg: Register, bits: u8) -> Result<(), E> {
        self.modify_reg(reg, |v| v | bits)
    }

    #[inline]
    fn reg_reset_bits(&mut self, reg: Register, bits: u8) -> Result<(), E> {
        self.modify_reg(reg, |v| v & !bits)
    }

    #[inline]
    fn _reg_xset_bits(&mut self, reg: Register, bits: u8, set: bool) -> Result<(), E> {
        if set {
            self.reg_set_bits(reg, bits)
        } else {
            self.reg_reset_bits(reg, bits)
        }
    }
}
