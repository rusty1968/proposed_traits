use proposed_traits::i2c_target::I2CTarget;

pub struct I2CTargetDriver<T: I2CTarget> {
    target: T,
}

impl<T: I2CTarget> I2CTargetDriver<T> {
    pub fn new(target: T) -> Self {
        Self { target }
    }

    pub fn handle_address_match(&mut self, addr: u8) -> bool {
        self.target.on_address_match(addr)
    }

    pub fn handle_transaction_start(&mut self, repeated: bool) {
        self.target.on_transaction_start(repeated);
    }

    pub fn handle_write(&mut self, data: &[u8]) -> Result<(), T::Error> {
        self.target.on_write(data)
    }

    pub fn handle_read(&mut self, buffer: &mut [u8]) -> Result<usize, T::Error> {
        self.target.on_read(buffer)
    }

    pub fn handle_stop(&mut self) {
        self.target.on_stop();
    }
}

/// Represents events that an I2C controller might generate for a target (slave) device.
pub enum I2CEvent<'a> {
    /// The controller sent an address; check if it matches.
    AddressMatch(u8),

    /// A new transaction has started.
    TransactionStart { repeated: bool },

    /// The controller is writing data to the target.
    Write(&'a [u8]),

    /// The controller is reading data from the target.
    Read(&'a mut [u8]),

    /// The controller issued a stop condition.
    Stop,
}

pub fn i2c_event_handler<T: I2CTarget>(driver: &mut I2CTargetDriver<T>, event: I2CEvent) {
    match event {
        I2CEvent::AddressMatch(addr) => {
            if driver.handle_address_match(addr) {
                // proceed
            }
        }
        I2CEvent::TransactionStart { repeated } => {
            driver.handle_transaction_start(repeated);
        }
        I2CEvent::Write(data) => {
            let _ = driver.handle_write(data);
        }
        I2CEvent::Read(buffer) => {
            let _ = driver.handle_read(buffer);
        }
        I2CEvent::Stop => {
            driver.handle_stop();
        }
    }
}
