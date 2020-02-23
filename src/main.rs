#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
extern crate embedded_hal as ehal;
extern crate panic_semihosting;
extern crate stm32l4xx_hal as hal;
extern crate ws2812_spi as ws2812;

use crate::ehal::spi::{Mode, Phase, Polarity, FullDuplex};
use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::hal::time::*;
use crate::hal::timer::*;
use crate::rt::ExceptionFrame;
use cortex_m::asm;
use crate::ws2812::prerendered::{Ws2812, Timing};
// use crate::ws2812::Ws2812;

use smart_leds::{brightness, SmartLedsWrite, RGB8};

use nb::block;

// SPI mode
pub const MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = hal::stm32::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    // TRY the other clock configuration
    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let clocks = rcc
        .cfgr
        .sysclk(80.mhz())
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .freeze(&mut flash.acr);

    let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);

    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    let mut spi = Spi::spi1(
        p.SPI1,
        (sck, miso, mosi),
        MODE,
        3.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    const NUM_LEDS: usize = 39;
    let mut delay = Delay::new(cp.SYST, clocks);
    let mut render_data = [0; MAX * 3 * 5];
    let mut data = [RGB8::default(); NUM_LEDS];
    let mut ws = Ws2812::new(spi, Timing::new(3000000).unwrap(), &mut render_data);

    loop {
        for j in 0..(256 * 5) {
            for i in 0..NUM_LEDS {
                data[i] = wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8);
            }
            ws.write(brightness(data.iter().cloned(), 32)).unwrap();
            delay.delay_ms(5u8);
        }
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
