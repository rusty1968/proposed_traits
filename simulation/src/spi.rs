use embedded_hal::spi::{ErrorKind, ErrorType, SpiBus};

#[derive(Debug, Default)]
pub struct SimulatedSpiBus {
    pub buffer: Vec<u8>,
    clock_freq: u32,
    polarity: bool,
    phase: bool,
}

impl SimulatedSpiBus {
    pub fn clock_freq(&self) -> u32 {
        self.clock_freq
    }
    pub fn set_clock_freq(&mut self, freq: u32) {
        self.clock_freq = freq;
    }
    pub fn polarity(&self) -> bool {
        self.polarity
    }
    pub fn set_polarity(&mut self, polarity: bool) {
        self.polarity = polarity;
    }
    pub fn phase(&self) -> bool {
        self.phase
    }
    pub fn set_phase(&mut self, phase: bool) {
        self.phase = phase;
    }
}

#[derive(Debug)]
pub struct SimulatedSpiBusError {
    kind: ErrorKind,
}

impl SimulatedSpiBusError {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl core::fmt::Display for SimulatedSpiBusError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Simulated pin error: {}", self.kind)
    }
}

impl embedded_hal::spi::Error for SimulatedSpiBusError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        self.kind
    }
}

impl ErrorType for SimulatedSpiBus {
    type Error = SimulatedSpiBusError;
}

impl<Word: Copy + 'static> SpiBus<Word> for SimulatedSpiBus {
    fn read(&mut self, _words: &mut [Word]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write(&mut self, _words: &[Word]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn transfer(&mut self, read: &mut [Word], write: &[Word]) -> Result<(), Self::Error> {
        // Simulate simultaneous read and write
        let len = read.len().min(write.len());
        for i in 0..len {
            read[i] = write[i]; // Replace with actual transfer logic
        }
        Ok(())
    }

    fn transfer_in_place(&mut self, _words: &mut [Word]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // Simulate flushing the SPI bus
        Ok(())
    }
}
