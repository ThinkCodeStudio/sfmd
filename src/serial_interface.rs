use core::fmt::Error;

pub trait SerialInterface {
    fn write(&mut self, cmd: &[u8])->Result<(), Error>;
    fn write_and_read(&mut self, cmd: &[u8], rev: &mut [u8])->Result<(), Error>;
    fn delay(&mut self, ms: u32);
}

// struct EmbeddedHalSPI<SPI, Pin>
// where
//     SPI: Transfer<u8> + Write<u8>,
//     Pin: OutputPin,
// {
//     spi: SPI,
//     cs_pin: Pin,
// }

// impl<SPI, Pin> EmbeddedHalSPI<SPI, Pin>
// where
//     SPI: Transfer<u8> + Write<u8>,
//     Pin: OutputPin,
// {
//     pub fn new(spi: SPI, cs_pin: Pin) -> Self {
//         Self { spi, cs_pin }
//     }
// }

// impl<SPI, Pin> SerialInterface for EmbeddedHalSPI<SPI, Pin>
// where
//     SPI: Transfer<u8> + Write<u8>,
//     Pin: OutputPin,
// {
//     fn write(&self, data: &[u8])->Result<(), Error> {
//         self.cs_pin.set_low().unwrap();
//         self.spi.write(data).unwrap();
//         self.cs_pin.set_high().unwrap();
//     }

//     fn read(&self, buffer: &mut [u8], len: usize)->Result<(), Error> {
//         self.cs_pin.set_low().unwrap();
//         self.spi.transfer(buffer).unwrap();
//         self.cs_pin.set_high().unwrap();
//     }
// }