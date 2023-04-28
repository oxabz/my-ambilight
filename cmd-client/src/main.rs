use std::time::SystemTime;
use std::{net::UdpSocket, time::Duration};

use rand::{prelude::Distribution, distributions::Uniform};
use udp_leds::constants::MAX_LED_COUNT;
use udp_leds::{constants::{MAX_MESSAGE_LENGTH, PORT}, client::ClientMessages, server::ServerMessages};

const PIXEL_COUNT: usize = 64;

fn gaussian(x: f64, mu: f64) -> f64 {
    (-(x - mu).powi(2)).exp()
}


fn main() {
    let mut rng = rand::thread_rng();
    let mut smessage = [0; 770];
    let mut cmessage: [u8; 770] = [0; MAX_MESSAGE_LENGTH];

    let udp = UdpSocket::bind(format!("0.0.0.0:{PORT}")).expect("Failed to bind to port");
    udp.set_broadcast(true).expect("Failed to set broadcast");
    udp.set_read_timeout(Some(Duration::from_millis(100))).expect("Failed to set read timeout");
    let mut input = String::new();

    let broadcast = std::net::SocketAddr::from(([255, 255, 255, 255], PORT));
    let mut server = broadcast.clone();
    let mut device = 0;

    loop {
        println!("Pick an action:");
        println!("[h]ello, [s]et active, [p]ixel, [r]gb, [R]ainbow!!!, [q]uit");

        input.clear();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        let i = input.trim();
        match i.chars().next().unwrap() {
            'h'=> {
                cmessage = ClientMessages::Hello.into();
                udp.send_to(&cmessage, broadcast).expect("Failed to send hello");
                while let Ok((size, addr)) = udp.recv_from(&mut smessage) {
                    if let Ok(ServerMessages::Hello) = ServerMessages::try_from(&smessage[..]) {
                        println!("Found server at {addr}");
                        server = addr;
                        break;
                    }
                }
            },
            's' => {
                device = Uniform::new(0, 64).sample(&mut rng);
                println!("Sending set active to device {}", device);
                cmessage = ClientMessages::set_active(device).into();
                udp.send_to(&cmessage, server).expect("Failed to send set active");
            },
            'p' => {
                input.clear();
                println!("Enter pixel number");
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
                let Ok(pixel) = input.trim().parse::<u8>() else {
                    println!("Invalid input");
                    continue;
                };
                input.clear();
                println!("Enter red value");
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
                let Ok(r) = input.trim().parse::<u8>() else {
                    println!("Invalid input");
                    continue;
                };
                input.clear();
                println!("Enter green value");
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
                let Ok(g) = input.trim().parse::<u8>() else {
                    println!("Invalid input");
                    continue;
                };
                input.clear();
                println!("Enter blue value");
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
                let Ok(b) = input.trim().parse::<u8>() else {
                    println!("Invalid input");
                    continue;
                };
                cmessage = ClientMessages::set_pixel(device, pixel, r, g, b).into();
                udp.send_to(&cmessage, server).expect("Failed to send set pixel");
            },
            'r' => {
                let start = SystemTime::now();
                let mut dur = Duration::from_secs(0);
                while dur < Duration::from_secs(15) {
                    let r = gaussian(dur.as_secs_f64(), 2.5) * 255.0;
                    let g = gaussian(dur.as_secs_f64(), 7.5) * 255.0;
                    let b = gaussian(dur.as_secs_f64(), 12.5) * 255.0;
                    let r = r as u8;
                    let g = g as u8;
                    let b = b as u8;
                    let mut pix = [0; MAX_LED_COUNT * 3];
                    for i in 0..PIXEL_COUNT {
                        pix[i * 3] = r;
                        pix[i * 3 + 1] = g;
                        pix[i * 3 + 2] = b;
                    }
                    cmessage = ClientMessages::send_pixels(device, pix).into();
                    udp.send_to(&cmessage, server).expect("Failed to send set pixel");
                    std::thread::sleep(Duration::from_millis(16));

                    dur = start.elapsed().unwrap();
                }
            },
            'q' => {
                break;
            },
            _ => {
                println!("Invalid input");
                continue;
            }
        }
    }
}
