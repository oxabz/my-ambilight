#![feature(generic_const_exprs)]
mod constants;
mod error;
pub mod leds;
mod logging;
mod wifi;

use std::net::UdpSocket;
use std::thread;

use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::rmt::config::{Loop, TransmitConfig};
use esp_idf_hal::rmt::{PinState, TxRmtDriver};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use log::{debug, error, info, warn};

const CLIENT_FLAG_BYTE: u8 = 0b0110_1011;
const SERVER_FLAG_BYTE: u8 = 0b1110_0110;
const INSTRUCTION_MASK: u8 = 0b11000000;
const DEVICE_ID_MASK: u8 = 0b000111111;

use log::{Level, Metadata, Record};

use crate::logging::SimpleLogger;

const fn is_hello(header: u8) -> bool {
    header & INSTRUCTION_MASK == 0b11000000
}

const fn is_set_active(header: u8) -> bool {
    header & INSTRUCTION_MASK == 0b01000000
}

fn main() {
    esp_idf_sys::link_patches();
    let sysloop = EspSystemEventLoop::take().unwrap();
    log::set_logger(&SimpleLogger).unwrap();
    info!("Starting up");

    // Retrieve the output pin and channel from peripherals.
    let peripherals = Peripherals::take().unwrap();
    let channel = peripherals.rmt.channel0;
    let rmt_pin = peripherals.pins.gpio32;
    let modem = peripherals.modem;
    debug!("Peripherals taken");

    let _wifi = wifi::setup_wifi(modem, sysloop);
    debug!("Wifi initialized");

    // Initializing the pixels
    let leds = leds::Leds::<{ constants::LED_COUNT as usize }>::new();
    debug!("Pixels initialized");

    // Initializing the rmt transmitter
    // Prepare the config.
    let config = TransmitConfig::new()
        .clock_divider(1)
        .idle(Some(PinState::Low))
        .looping(Loop::None);

    // Create the transmitter
    let rmt = TxRmtDriver::new(channel, rmt_pin, &config).unwrap();
    debug!("RMT initialized");

    std::thread::spawn({
        let leds = leds.clone();
        move || leds::led_update_loop(leds, rmt)
    });

    debug!("Thread created");

    let udp = UdpSocket::bind("0.0.0.0:52772").expect("Couldn't create the UDP socket");
    let mut buf: [u8; 780] = [0; 780];
    let mut active: u8 = 255;
    debug!("UDP initialized");

    info!("Initialization complete");

    loop {
        //std::thread::sleep(std::time::Duration::from_millis(1000));
        let (size, addr) = if let Ok((size, addr)) = udp.recv_from(&mut buf) {
            (size, addr)
        } else {
            error!("Error recieving a packet");
            continue;
        };
        println!("Recieved {} bytes from {}", size, addr);

        if size < 2 || buf[0] != CLIENT_FLAG_BYTE {
            warn!("Recieved a maleformed package");
            continue;
        }

        let header = buf[1];
        if is_hello(header) {
            debug!("Recieved a hello package");
            let resp = [SERVER_FLAG_BYTE, 0b1100_0000];
            udp.send_to(&resp, addr);
            continue;
        }

        let device = header & DEVICE_ID_MASK;
        if is_set_active(header) {
            println!("Recieved set active ({device})");
            active = device;
        }

        if device != active {
            println!("Recieved a package for a different device");
            continue;
        }

        let data = &buf[2..size];
        leds.set(data);
    }
}
