#![no_std]
#![no_main]

use arduino_uno::{Delay, prelude::*};
use arduino_uno::hal::i2c::Direction::Write;
use hd44780_driver::{HD44780, Cursor::*, CursorBlink};

use core::convert::TryInto;
use heapless::{String, consts::*};
use numtoa::NumToA;
use panic_halt as _;

const SHT30_ADDRESS: u8 = 0x44;  // SHT3x datasheet Page 9, Table 7.
const MEASURE_PERIODIC: [u8; 2] = [0x20, 0x32];   // Periodic Data Acquisition Mode. 0.5 mps. SHT3x datasheet Page 10, Table 9.
const READOUT: [u8; 2] = [0xE0, 0x00]; // Periodic Data Acquisition Mode. Readout. SHT3x datasheet Page 11, Table 10.

#[arduino_uno::entry]
fn main() -> ! {
    // Initialize board and pins
    let dp = arduino_uno::Peripherals::take().unwrap();

    let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);
    let mut serial = arduino_uno::Serial::new(
        dp.USART0,
        pins.d0,
        pins.d1.into_output(&mut pins.ddr),
        57600.into_baudrate(),
    );

    // Initialize i2c
    let mut i2c = arduino_uno::I2cMaster::new(
        dp.TWI,
        pins.a4.into_pull_up_input(&mut pins.ddr),  // * SDA
        pins.a5.into_pull_up_input(&mut pins.ddr),  // SCL
        50000,
    );

    i2c.start(SHT30_ADDRESS, Write).unwrap();  // Initialize SHT30 sensor.
    i2c.write(SHT30_ADDRESS, &MEASURE_PERIODIC).unwrap();  // Set measure mode.

    // Initialzie LCD. 
    let mut delay = Delay::new();

    let mut lcd = HD44780::new_4bit(
        pins.d8.into_output(&mut pins.ddr), // Register Select pin
        pins.d9.into_output(&mut pins.ddr), // Enable pin
        pins.d4.into_output(&mut pins.ddr),  // d4
        pins.d5.into_output(&mut pins.ddr),  // d5
        pins.d6.into_output(&mut pins.ddr),  // d6
        pins.d7.into_output(&mut pins.ddr),  // d7
        &mut delay,
    )
    .unwrap();

    lcd.reset(&mut delay).unwrap();
    lcd.set_cursor_visibility(Invisible, &mut delay).unwrap();
    lcd.set_cursor_blink(CursorBlink::Off, &mut delay).unwrap();

    let mut led = pins.d13.into_output(&mut pins.ddr); // Blinking LED.

    loop {

        led.toggle().void_unwrap();  // Blinking LED.
        
        // Read measurement results. SHT3x datasheet Page 10, Table 9.
        let mut buffer = [0u8; 6];
        i2c.write_read(SHT30_ADDRESS, &READOUT, &mut buffer).unwrap();

        let temp_msb = buffer[0];  // Temperature MSB
        let temp_lsb = buffer[1];  // Temperature LSB
        let hum_msb = buffer[3];  // Humidity MSB
        let hum_lsb = buffer[4];  // Humidity LSB

        let s_t: u32 = ((temp_msb as u16) * 256 + temp_lsb as u16).into();  // 16-bits temperature data.
        let temp: i16 = ((((s_t as i32) * 17500) >> 16) - 4500).try_into().unwrap(); // Temperature * 100 to get 2 digits decimal. SHT3x datasheet Page 13.
        let temp_int: u16 = ((temp / 100).abs()).try_into().unwrap();  // Integer part of temperature.
        let temp_dec: u16 = ((temp % 100).abs()).try_into().unwrap();  // Decimal part of temperature.

        let s_rh: u32 = ((hum_msb as u16) * 256 + hum_lsb as u16).into();  // 16-bits humidity data.
        let hum: u16 = ((s_rh * 10000) >> 16).try_into().unwrap();
        let hum_int: u16 = hum / 100;
        let hum_dec: u16 = hum % 100;

        // Add "-" if temperature < 0. 
        // Add 1 digit "0" if the decimal is 01, 02, 03, etc.
        
        // Output to serial port.
        if temp < 0 { 
            if temp_dec < 10 {
                ufmt::uwriteln!(&mut serial, "temp_MSB: {}, temp_LSB: {}, Temperature: -{}.0{} C.\r\n", 
                    temp_msb, temp_lsb, temp_int, temp_dec).void_unwrap();
            } else {
                ufmt::uwriteln!(&mut serial, "temp_MSB: {}, temp_LSB: {}, Temperature: -{}.{} C.\r\n", 
                    temp_msb, temp_lsb, temp_int, temp_dec).void_unwrap();
            }          
        } else {
            if temp_dec < 10 {
                ufmt::uwriteln!(&mut serial, "temp_MSB: {}, temp_LSB: {}, Temperature: {}.0{} C.\r\n", 
                    temp_msb, temp_lsb, temp_int, temp_dec).void_unwrap();
            } else {
                ufmt::uwriteln!(&mut serial, "temp_MSB: {}, temp_LSB: {}, Temperature: {}.{} C.\r\n", 
                    temp_msb, temp_lsb, temp_int, temp_dec).void_unwrap();
            }
            
        }
        
        if hum_dec < 10 {
            ufmt::uwriteln!(&mut serial, "hum_MSB: {}, hum_LSB: {}, Humidity: {}.0{} %RH.\r\n", 
                hum_msb, hum_lsb, hum_int, hum_dec).void_unwrap();
        } else {
            ufmt::uwriteln!(&mut serial, "hum_MSB: {}, hum_LSB: {}, Humidity: {}.{} %RH.\r\n", 
                hum_msb, hum_lsb, hum_int, hum_dec).void_unwrap();
        }

        // Display on LCD.
        let mut line_1 = [0u8; 20];
        let mut line_2 = [0u8; 20];       
        
        let mut display_line_1: String<U20> = String::new();
        let mut display_line_2: String<U20> = String::new();

        display_line_1.push_str("T: ").unwrap();

        if temp < 0 {
            display_line_1.push_str("-").unwrap();
        }

        display_line_1.push_str(temp_int.numtoa_str(10, &mut line_1)).unwrap();
        display_line_1.push_str(".").unwrap();

        if temp_dec < 10 {
            display_line_1.push_str("0").unwrap();
        }

        display_line_1.push_str(temp_dec.numtoa_str(10, &mut line_1)).unwrap();
        display_line_1.push_str(" C").unwrap();

        display_line_2.push_str("H: ").unwrap();
        display_line_2.push_str(hum_int.numtoa_str(10, &mut line_2)).unwrap();
        display_line_2.push_str(".").unwrap();

        if hum_dec < 10 {
            display_line_2.push_str("0").unwrap();
        }

        display_line_2.push_str(hum_dec.numtoa_str(10, &mut line_2)).unwrap();
        display_line_2.push_str(" %RH").unwrap();

        lcd.write_str(&display_line_1, &mut delay).unwrap();
        lcd.set_cursor_pos(40, &mut delay).unwrap();  // Go to line 2.
        lcd.write_str(&display_line_2, &mut delay).unwrap();

        arduino_uno::delay_ms(4000);

        lcd.clear(&mut delay).unwrap();
    }
}