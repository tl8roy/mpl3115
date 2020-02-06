#![allow(non_upper_case_globals)]

/// I2C slave address
pub const I2C_SAD: u8 = 0x60; //MPL3115A2_ADDRESS

/// Register mapping
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Register {
    STATUS = 0x00,
    OUT_P_MSB = 0x01,
    OUT_P_CSB = 0x02,
    OUT_P_LSB = 0x03,
    OUT_T_MSB = 0x04,
    OUT_T_LSB = 0x05,
    DR_STATUS = 0x06,
    OUT_P_DELTA_MSB = 0x07,
    OUT_P_DELTA_CSB = 0x08,
    OUT_P_DELTA_LSB = 0x09,
    OUT_T_DELTA_MSB = 0x0A,
    OUT_T_DELTA_LSB = 0x0B,
    WHO_AM_I  = 0x0C,
    F_STATUS  = 0x0D,
    F_DATA    = 0x0E,
    F_SETUP   = 0x0F,
    TIME_DLY  = 0x10,
    SYSMOD    = 0x11,
    INT_SOURCE = 0x12,
    PT_DATA_CFG = 0x13,
    BAR_IN_MSB = 0x14,
    BAR_IN_LSB = 0x15,
    P_TGT_MSB = 0x16,
    P_TGT_LSB = 0x17,
    T_TGT     = 0x18,
    P_WND_MSB = 0x19,
    P_WND_LSB = 0x1A,
    T_WND     = 0x1B,
    P_MIN_MSB = 0x1C,
    P_MIN_CSB = 0x1D,
    P_MIN_LSB = 0x1E,
    T_MIN_MSB = 0x1F,
    T_MIN_LSB = 0x20,
    P_MAX_MSB = 0x21,
    P_MAX_CSB = 0x22,
    P_MAX_LSB = 0x23,
    T_MAX_MSB = 0x24,
    T_MAX_LSB = 0x25,
    CTRL_REG1 = 0x26,
    CTRL_REG2 = 0x27,
    CTRL_REG3 = 0x28,
    CTRL_REG4 = 0x29,
    CTRL_REG5 = 0x2A,
    OFF_P     = 0x2B,
    OFF_T     = 0x2C,
    OFF_H     = 0x2D,

}

impl Register {
    /// Get register address
    pub fn addr(self) -> u8 {
        self as u8
    }
}


//WHO_AM_I device identification value
pub const DEVICE_ID: u8 = 0xC4; //WHO_AM_I

//CTRL_REG1
//SBYB bit for Constant Measurement
pub const DEVICE_EN: u8 = 0b0000_0001;

//One Shot Trigger bit
pub const ONE_SHOT: u8 = 0b0000_0010;

//OS bits
//pub const OVERSAMPLE: u8 = 0b0011_1000;

//ALT Enable bit
pub const ALT_EN: u8 = 0b1000_0000;

//STATUS
//Temp Data Ready
pub const TDR: u8 = 0b0000_0010;
//Pressure Data Ready
pub const PDR: u8 = 0b0000_0100;

//PT_DATA_CFG
//Event Flags Enable
pub const EVENT_FLAGS: u8 = 0x07;




