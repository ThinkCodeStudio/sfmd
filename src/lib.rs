#![no_std]

use core::fmt::Error;

pub mod define;
pub mod flash;
pub mod serial_interface;
pub mod sfdp;

pub struct FlashInfo {
    manufacturer_id: u8,
    type_id: u8,
    capacity_id: u8,
    capacity: usize,
    secter_size: u32,
}

impl FlashInfo {
    pub fn new(manufacturer_id: u8, type_id: u8, capacity_id: u8, capacity: usize, secter_size: u32) -> Self {
        FlashInfo {
            manufacturer_id,
            type_id,
            capacity_id,
            capacity,
            secter_size,
        }
    }
}

pub trait FlashOperations {
    fn erase_chip(&mut self) -> Result<(), Error>;
    fn erase(&mut self, address: u32, size: usize) -> Result<(), Error>;
    fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), Error>;
    fn read_data(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), Error>;
    fn read_status(&mut self) -> Result<u8, Error>;
    fn write_state(&mut self, is_volatile: bool, state: u8) -> Result<(), Error>;
}
