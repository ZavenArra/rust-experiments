//! Serial interface loopback test
//!
//! You have to short the TX and RX pins to make this program work

#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

extern crate alloc;

use core::{default::Default, u8};

use alloc::{boxed::Box, vec::{self, Vec}};
use panic_halt as _;

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac,
    gpio,
    prelude::*,
    serial::{Config, Serial},
    serial::*
};
use unwrap_infallible::UnwrapInfallible;

#[no_mangle]
pub static CR: u8 = b'\r';
#[no_mangle]
pub static LF: u8 = b'\n';

pub struct SerialInterfaceContext {
    rx: Rx<pac::USART2>,
    tx: Tx<pac::USART2>,
    chars: Vec<u8>,
}

#[no_mangle]
pub unsafe extern "C" fn rust_serial_interface_new() -> *mut SerialInterfaceContext {
    // Get access to the device specific peripherals from the peripheral access crate
    let p = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = p.FLASH.constrain();
    let rcc = p.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Prepare the alternate function I/O registers
    let mut afio = p.AFIO.constrain();

    // Prepare the GPIOB peripheral
    let mut gpioa = p.GPIOA.split();

    // USART1
    // let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    // let rx = gpioa.pa10;

    // USART1
    // let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    // let rx = gpiob.pb7;

    // USART2
    let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx = gpioa.pa3;

    // // USART3
    // // Configure pb10 as a push_pull output, this will be the tx pin
    // let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    // // Take ownp.USART2usart device. Take ownership over the USART register and tx/rx pins. The rest of
    // the registers are used to enable and configure the device.
    // let mut SERIAL:Serial<stm32f1xx_hal::pac::USART2, (stm32f1xx_hal::gpio::Pin<'A', 2, stm32f1xx_hal::gpio::Alternate>, stm32f1xx_hal::gpio::Pin<'A', 3>)> = Serial::new(
    let serial = Serial::new(
        p.USART2,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(9600.bps()),
        &clocks,
    );
    let context = SerialInterfaceContext {
        rx: serial.rx,
        tx: serial.tx,
        chars: Vec::new(),
    };
    Box::into_raw(Box::new(context))
}

#[no_mangle]
pub unsafe extern "C" fn rust_serial_read(serial_ptr: *mut SerialInterfaceContext) -> u8 {
    
    let mut serial_context = Box::from_raw(serial_ptr);
    let rx = &mut serial_context.rx;
    // Read the byte that was just sent. Blocks until the read is complete
    block!(rx.read()).unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn rust_serial_write(serial_ptr: *mut SerialInterfaceContext, value: u8) {
    let mut serial_context = Box::from_raw(serial_ptr);
    let tx = &mut serial_context.tx;
        block!(tx.write(value)).unwrap_infallible();
        block!(tx.write(LF)).unwrap_infallible();
        block!(tx.write(CR)).unwrap_infallible();
}
