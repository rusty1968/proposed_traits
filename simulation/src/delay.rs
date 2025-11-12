use std::{thread, time::Duration};

use embedded_hal::delay::DelayNs;

#[derive(Default)]
pub struct SimulatedDelay;

impl DelayNs for SimulatedDelay {
    #[inline]
    fn delay_ns(&mut self, ns: u32) {
        let us = ns.saturating_add(999) / 1_000; // Convert nanoseconds to microseconds
        self.delay_us(us);
    }

    #[inline]
    fn delay_us(&mut self, us: u32) {
        thread::sleep(Duration::from_micros(us as u64));
    }

    #[inline]
    fn delay_ms(&mut self, ms: u32) {
        thread::sleep(Duration::from_millis(ms as u64));
    }
}
