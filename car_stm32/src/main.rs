#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _; // Panic handler

use stm32f4xx_hal as hal;

use hal::{
    pac,
    prelude::*,
    serial::{config::Config, Serial},
};

use core::fmt::Write;
use rtt_target::{rtt_init_print, rprintln};

#[entry]
fn main() -> ! {
    // Take ownership of the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Set up the system clock. We are using the default settings here.
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    // Acquire the GPIOA peripheral
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();

    // Set up USART2 at PA2 (TX) and PA3 (RX)
    let tx_pin = gpioa.pa2.into_alternate();
    let rx_pin = gpioa.pa3.into_alternate();

    // Configure the serial peripheral
    let serial = Serial::new(
        dp.USART2,
        (tx_pin, rx_pin),
        Config::default().baudrate(115_200.bps()),
        &clocks, // Pass a reference to clocks
    )
    .unwrap();

    let (pwm_manager, (pwm_ch1, pwm_ch2, pwm_ch3, pwm_ch4)) = dp.TIM3.pwm_us(20.millis(), &clocks);
    let mut pwm_c1 = pwm_ch1.with(gpioc.pc6);
    pwm_c1.enable();

    let max = pwm_c1.get_max_duty();
    pwm_c1.set_duty(max / 10);


    let (mut tx, _rx) = serial.split();

    // Send "Hello, world!" over serial
    writeln!(tx, "Hello, world!\r").unwrap();

    // Enter an infinite loop to prevent the program from exiting
    rtt_init_print!();
    rprintln!("Hello, world!");
    let mut i = 0;
    loop {
        rprintln!("LOOP {}", max);
        i += 1;
    }
}
