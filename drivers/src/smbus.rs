use embedded_hal::i2c::{I2c, SevenBitAddress};

pub struct Smbus<I2C> {
    i2c: I2C,
}

impl<I2C, E> Smbus<I2C>
where
    I2C: I2c<SevenBitAddress, Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Smbus { i2c }
    }

    pub fn write_byte(&mut self, addr: u8, data: u8) -> Result<(), E> {
        self.i2c.write(addr, &[data])
    }

    pub fn read_byte(&mut self, addr: u8) -> Result<u8, E> {
        let mut buf = [0];
        self.i2c.write_read(addr, &[], &mut buf)?;
        Ok(buf[0])
    }

    pub fn write_read(&mut self, addr: u8, data: &[u8], buffer: &mut [u8]) -> Result<(), E> {
        self.i2c.write_read(addr, data, buffer)
    }
}
