#![no_std]
#![no_main]

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

        let temp_int = (((reading as i16 * 5) - 750) / 10 ) + 25;
        let temp_dec = (reading as i16 * 5) % 10;

        let mut buf = [0u8; 20];    
        let mut s_int: String<U8> = String::new();
        let mut s_dec: String<U8> = String::new();
 
        s_int.push_str(temp_int.numtoa_str(10, &mut buf)).unwrap();
        s_dec.push_str(temp_dec.numtoa_str(10, &mut buf)).unwrap();
        
        lcd.write_str("T: ", &mut delay).unwrap();
        lcd.write_str(&s_int, &mut delay).unwrap();
        lcd.write_str(".", &mut delay).unwrap();
        lcd.write_str(&s_dec, &mut delay).unwrap();
        lcd.write_str(" C", &mut delay).unwrap();

        led.toggle().void_unwrap();

        arduino_uno::delay_ms(2000);

        lcd.clear(&mut delay).unwrap();

    }
}
