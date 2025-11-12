use simulation::delay::SimulatedDelay;
use simulation::SimulatedPac;

use drivers::spi_device_driver::SpiDeviceDriver;


pub fn main() {
    let pac = SimulatedPac::new();
    let config = drivers::spi_device_driver::Config::default();
    let mut driver = SpiDeviceDriver::new(
        config,
        pac.spi_master,
        pac.cs_pin,
        pac.busy_pin,
        pac.reset_pin,
        SimulatedDelay::default(),
    );
    driver.init().unwrap();

    driver.write(&[1, 2, 3]).unwrap();
}
