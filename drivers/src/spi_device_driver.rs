use core::marker::PhantomData;
use embedded_hal::delay;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiBus as SpiMaster;

#[derive(Debug, Clone, PartialEq)]
pub enum Error<SpiError, PinError> {
    /// Underlying SPI device error
    Spi(SpiError),
    /// Underlying GPIO pin error
    Pin(PinError),

    /// Device failed to resume from reset
    BusyTimeout,
}

pub struct SpiDeviceDriver<Spi, CsPin, BusyPin, ResetPin, Delay, SpiError, PinError> {
    spi: Spi,
    cs: CsPin,
    busy: BusyPin,
    reset: ResetPin,
    delay: Delay,
    config: Config,
    _spi_err: PhantomData<SpiError>,
    _pin_err: PhantomData<PinError>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Config {
    poll_interval: u32,
}

impl<Spi, CsPin, BusyPin, ResetPin, Delay, SpiError, PinError>
    SpiDeviceDriver<Spi, CsPin, BusyPin, ResetPin, Delay, SpiError, PinError>
where
    // define associated types as generic parameters
    CsPin: OutputPin<Error = PinError>,
    Spi: SpiMaster<Error = SpiError>,
    BusyPin: InputPin<Error = PinError>,
    ResetPin: OutputPin<Error = PinError>,
    Delay: delay::DelayNs,
{
    pub fn new(
        config: Config,
        spi: Spi,
        cs: CsPin,
        busy: BusyPin,
        reset: ResetPin,
        delay: Delay,
    ) -> Self {
        Self {
            spi,
            cs,
            busy,
            reset,
            delay,
            config,
            _spi_err: PhantomData,
            _pin_err: PhantomData,
        }
    }

    pub fn init(&mut self) -> Result<(), Error<SpiError, PinError>> {
        self.reset()?;
        self.wait_busy(1000)?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Error<SpiError, PinError>> {
        self.reset.set_high().map_err(Error::Pin)?;
        self.delay.delay_ms(1);
        self.reset.set_low().map_err(Error::Pin)?;

        Ok(())
    }

    pub fn wait_busy(&mut self, timeout_ms: u32) -> Result<(), Error<SpiError, PinError>> {
        let mut elapsed = 0;

        while self.busy.is_high().map_err(Error::Pin)? {
            if elapsed >= timeout_ms {
                return Err(Error::BusyTimeout);
            }
            self.delay.delay_ms(self.config.poll_interval);
            elapsed += 1;
        }

        Ok(())
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), Error<SpiError, PinError>> {
        // Assert the CS line to select the slave device
        self.cs.set_low().map_err(Error::Pin)?;
        // Send data to the slave device
        self.spi.write(data).map_err(Error::Spi)?;
        // Deassert the CS line to deselect the slave device
        self.cs.set_high().map_err(Error::Pin)?;

        Ok(())
    }
}
