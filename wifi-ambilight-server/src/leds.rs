use esp_idf_hal::{
    rmt::{FixedLengthSignal, PinState, Pulse, TxRmtDriver},
    units::Hertz,
};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

const T0H: Duration = Duration::from_nanos(350);
const T1H: Duration = Duration::from_nanos(700);
const T0L: Duration = Duration::from_nanos(800);
const T1L: Duration = Duration::from_nanos(600);
const RESET: Duration = Duration::from_millis(1);

#[derive(Debug, Clone)]
pub struct Leds<const L: usize>
where
    [(); L * 3]:,
    [(); L * 3 * 8]:,
{
    pixels: Arc<Mutex<[u8; L * 3]>>,
}

impl<const L: usize> Leds<L>
where
    [(); L * 3]:,
    [(); L * 3 * 8]:,
{
    pub fn new() -> Self {
        Self {
            pixels: Arc::new(Mutex::new([255; L * 3])),
        }
    }

    pub fn set(&self, bytes: &[u8]) {
        let mut pixels = self.pixels.lock().unwrap();
        pixels.copy_from_slice(&bytes[..{ L * 3 }]);
    }

    pub fn to_rmt_signal(&self, freq: Hertz) -> FixedLengthSignal<{ L * 3 * 8 }> {
        let mut signal = FixedLengthSignal::new();
        let one = (
            Pulse::new_with_duration(freq, PinState::High, &T1H).unwrap(),
            Pulse::new_with_duration(freq, PinState::Low, &T1L).unwrap(),
        );
        let zero = (
            Pulse::new_with_duration(freq, PinState::High, &T0H).unwrap(),
            Pulse::new_with_duration(freq, PinState::Low, &T0L).unwrap(),
        );
        let pixels = self.pixels.lock().unwrap();
        for (i, byte) in pixels.iter().enumerate() {
            for bit in 0..8 {
                let bit = byte & (1 << bit) != 0;
                let pair = if bit { one } else { zero };
                signal.set(i * 8 + bit as usize, &pair);
            }
        }
        signal
    }
}

impl<const L: usize> Default for Leds<L>
where
    [(); L * 3]:,
    [(); L * 3 * 8]:,
{
    fn default() -> Self {
        Self::new()
    }
}

pub fn led_update_loop<const L: usize>(leds: Leds<L>, rmt: TxRmtDriver) -> !
where
    [(); L * 3]:,
    [(); L * 3 * 8]:,
{
    let mut rmt = rmt;
    let freq = rmt.counter_clock().unwrap();
    loop {
        let signal = leds.to_rmt_signal(freq);
        rmt.start_blocking(&signal).unwrap();
        std::thread::sleep(Duration::from_millis(1000));
    }
}
