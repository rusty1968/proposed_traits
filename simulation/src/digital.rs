use embedded_hal::digital::{ErrorKind, ErrorType, InputPin, OutputPin};
use rand::Rng;

#[derive(Debug)]
pub struct SimulatedPinError {
    kind: ErrorKind,
}

impl SimulatedPinError {
    pub fn new(kind: ErrorKind) -> Self {
        SimulatedPinError { kind }
    }
}

impl core::fmt::Display for SimulatedPinError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Simulated pin error: {}", self.kind)
    }
}

pub struct SimulatedOutputPin {
    state: bool,
}

impl SimulatedOutputPin {
    pub fn new() -> Self {
        SimulatedOutputPin { state: false }
    }
}

impl embedded_hal::digital::Error for SimulatedPinError {
    fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl ErrorType for SimulatedOutputPin {
    type Error = SimulatedPinError;
}

impl OutputPin for SimulatedOutputPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.state = false;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.state = true;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SimulatedInputPin;

impl ErrorType for SimulatedInputPin {
    type Error = SimulatedPinError;
}

impl InputPin for SimulatedInputPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        let mut rng = rand::thread_rng();
        let random_state: bool = rng.gen();
        Ok(random_state)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        let mut rng = rand::thread_rng();
        let random_state: bool = rng.gen();
        Ok(random_state)
    }
}
