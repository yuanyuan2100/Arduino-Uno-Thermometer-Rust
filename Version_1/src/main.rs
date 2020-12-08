#![no_std]
#![no_main]
#![allow(arithmetic_overflow)]

use arduino_uno::prelude::*;
use arduino_uno::{adc, Delay};
use hd44780_driver::HD44780;
use hd44780_driver::{Cursor::*, CursorBlink};
use panic_halt as _;

use heapless::String;
use heapless::consts::*;
use numtoa::NumToA;

#[arduino_uno::entry]
fn main() -> ! {
    let dp = arduino_uno::Peripherals::take().unwrap();

    let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

    let mut serial =
        arduino_uno::Serial::new(dp.USART0, pins.d0, pins.d1.into_output(&mut pins.ddr), 9600.into_baudrate());

    let mut adc = adc::Adc::new(dp.ADC, Default::default());
    let mut a0 = pins.a0.into_analog_input(&mut adc);

    let mut delay = Delay::new();

    let mut lcd = HD44780::new_4bit(
        pins.d12.into_output(&mut pins.ddr), // Register Select pin
        pins.d11.into_output(&mut pins.ddr), // Enable pin
        pins.d2.into_output(&mut pins.ddr),  // d4
        pins.d3.into_output(&mut pins.ddr),  // d5
        pins.d4.into_output(&mut pins.ddr),  // d6
        pins.d5.into_output(&mut pins.ddr),  // d7
        &mut delay,
    )
    .unwrap();

    lcd.reset(&mut delay).unwrap();
    lcd.clear(&mut delay).unwrap();
    lcd.set_cursor_visibility(Invisible, &mut delay).unwrap();
    lcd.set_cursor_blink(CursorBlink::Off, &mut delay).unwrap();

    let mut led = pins.d13.into_output(&mut pins.ddr);
    
    loop {
        let reading: u16 = nb::block!(adc.read(&mut a0)).void_unwrap();

        // u16 overflowed. i32 seems not work. So it has to be 25000 - (75000 - 65536)
        // The original formula should be (reading * 488 - 75000) + 25000
        let temp = (reading as i16 * 488 + 15536) / 10; 
                                                 
        let temp_int = (temp / 100).abs();
        let temp_dec = (temp % 100).abs();

        if reading < 103 {
            ufmt::uwrite!(&mut serial, "Temp: -{}.{} C \n\r", temp_int, temp_dec).void_unwrap();
        } else {
            ufmt::uwrite!(&mut serial, "Temp: {}.{} C \n\r", temp_int, temp_dec).void_unwrap();
        }

        let mut buf = [0u8; 20];    
        
        let mut display: String<U20> = String::new();

        display.push_str("T: ").unwrap();
        if reading < 103 {
            display.push_str("-").unwrap();
        }
        display.push_str(temp_int.numtoa_str(10, &mut buf)).unwrap();
        display.push_str(".").unwrap();
        display.push_str(temp_dec.numtoa_str(10, &mut buf)).unwrap();
        display.push_str(" C").unwrap();

        lcd.write_str(&display, &mut delay).unwrap();

        led.toggle().void_unwrap();

        arduino_uno::delay_ms(2000);

        lcd.clear(&mut delay).unwrap();

    }
}
