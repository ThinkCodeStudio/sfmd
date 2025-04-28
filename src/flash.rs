use core::fmt::Error;

use log::{error, info};

use crate::serial_interface::SerialInterface;
use crate::{FlashInfo, FlashOperations, define};
const PAGE_SIZE: usize = 256;

/// Flash struct
/// I - SerialInterface
/// C - Flash capacity
/// S - Flash sector size
///
pub struct Flash<I>
where
    I: SerialInterface,
{
    flash_info: FlashInfo,
    interface: I,
    enable_address_4_byte: bool,
}

impl<I> Flash<I>
where
    I: SerialInterface,
{
    pub fn new(interface: I, flash_info: FlashInfo) -> Result<Self, Error> {
        let capacity = flash_info.capacity;
        let mut flash = Flash {
            flash_info,
            interface: interface,
            enable_address_4_byte: if capacity > (1 << 24) { true } else { false },
        };

        let mut jedec_id = [0_u8; 3];

        if flash.read_jedec_id(&mut jedec_id).is_err() {
            return Err(Error);
        }

        if flash.flash_info.manufacturer_id != jedec_id[0]
            || flash.flash_info.type_id != jedec_id[1]
            || flash.flash_info.capacity_id != jedec_id[2]
        {
            error!(
                "JEDEC ID mismatch: expected {:02X} {:02X} {:02X}, got {:02X} {:02X} {:02X}",
                flash.flash_info.manufacturer_id,
                flash.flash_info.type_id,
                flash.flash_info.capacity_id,
                jedec_id[0],
                jedec_id[1],
                jedec_id[2]
            );
            return Err(Error);
        }

        // reset()

        if flash.write_state(true, 0x00).is_err() {
            return Err(Error);
        }

        if flash.set_4byte_address_mode().is_err() {
            return Err(Error);
        }

        Ok(flash)
    }

    // TODO build from SFDP
    // pub fn build(SFDP)->{

    // }

    // fn reset(&self){
    //     let cmd = [cmd::MODE_CMD::MODE_RESET as u8];
    // }

    fn read_jedec_id(&mut self, buff: &mut [u8]) -> Result<(), Error> {
        // Read JEDEC ID
        let cmd = [define::IdCmd::JedecId as u8];
        if self.interface.write_and_read(&cmd, buff).is_ok() {
            info!("JEDEC ID: {:02X} {:02X} {:02X}", buff[0], buff[1], buff[2]);
            Ok(())
        } else {
            error!("Failed to read JEDEC ID");
            Err(Error)
        }
    }

    fn write_enable(&mut self, enable: bool) -> Result<(), Error> {
        // Write Enable
        let cmd = if enable {
            [define::WriteCmd::WriteEnable as u8]
        } else {
            [define::WriteCmd::WriteDisable as u8]
        };

        if self.interface.write(&cmd).is_err() {
            error!("Failed to write enable");
            return Err(Error);
        }
        if let Ok(status) = self.wait_busy() {
            if enable && (status & define::STATUS::WEL as u8) == 0 {
                error!("Write enable failed status: {:02X}", status);
                Err(Error)
            } else if !enable && (status & define::STATUS::WEL as u8) != 0 {
                error!("Write disable failed status: {:02X}", status);
                Err(Error)
            } else {
                Ok(())
            }
        } else {
            error!("Failed to wait for write enable operation to complete");
            Err(Error)
        }
    }

    fn write_operation<F: FnOnce(&mut Self) -> Result<(), Error>>(
        &mut self,
        operation: F,
    ) -> Result<(), Error> {
        let ret = if self.write_enable(true).is_ok() {
            operation(self)
        } else {
            Err(Error)
        };
        let _ = self.write_enable(false);
        return ret;
    }

    fn wait_busy(&mut self) -> Result<u8, Error> {
        // Wait for the flash to be ready
        let mut is_ok = false;
        let mut ret_status = 0_u8;
        for _ in 0..50 {
            if let Ok(status) = self.read_status() {
                if (status & define::STATUS::BUSY as u8) == 0 {
                    is_ok = true;
                    ret_status = status;
                    break;
                } else {
                    self.interface.delay(10);
                }
            } else {
                return Err(Error);
            }
        }

        if is_ok {
            Ok(ret_status)
        } else {
            error!("Flash is busy for too long");
            Err(Error)
        }
    }

    fn set_4byte_address_mode(&mut self) -> Result<(), Error> {
        // Set 4-byte address mode
        self.write_operation(|s| {
            let cmd = if s.enable_address_4_byte {
                [0xB7]
            } else {
                [0xE9]
            };
            if s.interface.write(&cmd).is_err() {
                error!("Failed to set 4-byte address mode");
                return Err(Error);
            }
            Ok(())
        })
    }

    const fn address_len(&self) -> usize {
        if self.enable_address_4_byte { 4 } else { 3 }
    }

    fn make_address_byte_array(&self, address: u32, buff: &mut [u8]) {
        let len = self.address_len();
        for i in 0..len {
            buff[i] = (address >> ((len - (i + 1)) * 8)) as u8;
        }
    }

    fn page_write(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        // Page Program
        if data.len() > PAGE_SIZE {
            error!("Data size exceeds {} bytes", PAGE_SIZE);
            return Err(Error);
        }

        self.write_operation(|s| {
            let mut cmd = [define::WriteCmd::PageProgram as u8, 0, 0, 0, 0];
            s.make_address_byte_array(address, &mut cmd[1..]);
            let cmd_len = s.address_len() + 1;

            if s.interface.write(&cmd[..cmd_len]).is_err() {
                return Err(Error);
            }
            if s.interface.write(data).is_err() {
                return Err(Error);
            }
            if s.wait_busy().is_err() {
                return Err(Error);
            }
            Ok(())
        })
    }
}

impl<I> FlashOperations for Flash<I>
where
    I: SerialInterface,
{
    fn erase_chip(&mut self) -> Result<(), Error> {
        self.write_operation(|s| {
            let cmd = [define::EraseCmd::Chip as u8];
            s.interface.write(&cmd)
        })
    }

    fn erase(&mut self, address: u32, size: usize) -> Result<(), Error> {
        assert!(
            size % self.flash_info.secter_size as usize == 0,
            "erase_size must be secter_size"
        );
        assert!(
            address % self.flash_info.secter_size == 0,
            "address must be secter_size aligned"
        );

        if (address + size as u32) > self.flash_info.capacity as u32 {
            return Err(Error);
        }
        if address == 0 && size == self.flash_info.capacity as usize {
            return self.erase_chip();
        }

        self.write_operation(|s| {
            let mut size = size;
            let mut cmd = [define::EraseCmd::Block64k as u8, 0, 0, 0, 0];
            let mut addr = address;
            while size > 0 {
                s.make_address_byte_array(addr, &mut cmd[1..]);
                let cmd_len = s.address_len() + 1;
                if s.interface.write(&cmd[..cmd_len]).is_err() {
                    error!("Failed to erase block at address {:08X}", addr);
                    return Err(Error);
                }
                if s.wait_busy().is_err() {
                    error!("Failed to wait for erase operation to complete");
                    return Err(Error);
                }
                if (addr % s.flash_info.secter_size as u32) != 0 {
                    if size
                        > s.flash_info.secter_size as usize
                            - (addr % s.flash_info.secter_size as u32) as usize
                    {
                        size -= s.flash_info.secter_size as usize
                            - (addr % s.flash_info.secter_size as u32) as usize;
                        addr += s.flash_info.secter_size as u32
                            - (addr % s.flash_info.secter_size as u32) as u32;
                    } else {
                        return Ok(());
                    }
                } else {
                    if size > s.flash_info.secter_size as usize {
                        size -= s.flash_info.secter_size as usize;
                        addr += s.flash_info.secter_size as u32;
                    } else {
                        return Ok(());
                    }
                }
            }
            Ok(())
        })
    }

    fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        let mut data_len = data.len();
        let mut offset = 0_usize;
        let get_send_data_len = |len: &mut usize| {
            if *len > PAGE_SIZE { PAGE_SIZE } else { *len }
        };
        loop {
            let send_data_len = get_send_data_len(&mut data_len);
            if self
                .page_write(address + offset as u32, &data[offset..offset + send_data_len])
                .is_err()
            {
                error!(
                    "Failed to write data to address {:08X}",
                    address + offset as u32
                );
                return Err(Error);
            }

            offset += send_data_len;
            data_len -= send_data_len;

            if data_len == 0 {
                break;
            }
        }
        Ok(())
    }

    fn read_data(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), Error> {
        if address + buffer.len() as u32 > self.flash_info.capacity as u32 {
            error!(
                "Read out of bounds: address {:08X} + size {} > flash size {}",
                address,
                buffer.len(),
                self.flash_info.capacity
            );
            return Err(Error);
        }

        if self.wait_busy().is_err() {
            return Err(Error);
        }

        let mut cmd = [define::ReadCmd::Data as u8, 0, 0, 0, 0];
        self.make_address_byte_array(address, &mut cmd[1..]);
        let cmd_len = self.address_len() + 1;

        if self
            .interface
            .write_and_read(&cmd[..cmd_len], buffer)
            .is_err()
        {
            error!("Failed to read data from address {:08X}", address);
            return Err(Error);
        }

        Ok(())
    }

    fn read_status(&mut self) -> Result<u8, Error> {
        let mut buff = [0_u8; 1];
        let cmd = [define::ReadCmd::Status1 as u8];

        if self.interface.write_and_read(&cmd, &mut buff).is_ok() {
            Ok(buff[0])
        } else {
            error!("Failed to read status register");
            Err(Error)
        }
    }

    fn write_state(&mut self, is_volatile: bool, state: u8) -> Result<(), Error> {
        self.write_operation(|s| {
            let cmd = [define::WriteCmd::WrietStatus as u8, state];
            if s.interface.write(&cmd).is_err() {
                error!("Failed to write status register");
                return Err(Error);
            }
            Ok(())
        })
    }
}
