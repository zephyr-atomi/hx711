//! HX 711 Library
//!
//! This prints "Interrupt" when the boot button is pressed.
//! It also blinks an LED like the blinky example.

#![no_std]
#![no_main]

use core::{borrow::BorrowMut, cell::RefCell};

use critical_section::Mutex;
use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::ClockControl,
    gpio::IO,
    interrupt,
    peripherals::{self, Peripherals},
    prelude::*,
    Delay,
};

mod hx711;

static HX711_MUTEX: Mutex<RefCell<Option<hx711::HX711<4, 16>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.DPORT.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Set GPIO15 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let hx711_sck = io.pins.gpio4.into_push_pull_output();

    let hx711_dt = io.pins.gpio16.into_floating_input();

    let mut delay = Delay::new(&clocks);
    let mut load_sensor = hx711::HX711::new(hx711_sck, hx711_dt, &delay);

    load_sensor.tare();
    println!("Tare = {}", load_sensor.get_offset());

    critical_section::with(|cs| {
        HX711_MUTEX.borrow_ref_mut(cs).replace(load_sensor);
    });

    interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority2).unwrap();

    loop {
        critical_section::with(|cs| {
            println!(
                "Last Reading = {}",
                HX711_MUTEX.borrow_ref_mut(cs).as_mut().unwrap().get_last()
            );
        });

        delay.delay_ms(50u32);
    }
}

#[ram]
#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        let mut bind = HX711_MUTEX.borrow_ref_mut(cs);
        let hx711 = bind.borrow_mut().as_mut().unwrap();
        if hx711.is_ready() {
            hx711.read();
        }
    });
}
