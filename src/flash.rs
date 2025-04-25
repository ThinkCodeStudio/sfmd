use core::fmt::Error;

use log::{error, info};

use crate::serial_interface::SerialInterface;
use crate::{FlashInfo, FlashOperations, define};

pub struct Flash<I, const SIZE: usize>
where
    I: SerialInterface,
{
    flash_info: FlashInfo,
    buffer: [u8; SIZE],
    interface: I,
    init_ok: bool,
    addr_4byte: bool,
}

impl<I, const SIZE: usize> Flash<I, SIZE>
where
    I: SerialInterface,
{
    pub fn new(interface: I, flash_info: FlashInfo) -> Self {
        Flash {
            flash_info,
            buffer: [0; SIZE],
            interface: interface,
            init_ok: false,
            addr_4byte: false,
        }
    }

    // TODO build from SFDP
    // pub fn build(SFDP)->{

    // }

    pub fn init(&mut self) -> Result<(), Error> {
        if self.init_ok {
            return Ok(());
        }

        if self.read_jedec_id().is_err() {
            return Err(Error);
        }

        if self.flash_info.manufacturer_id != self.buffer[0]
            || self.flash_info.type_id != self.buffer[1]
            || self.flash_info.capacity_id != self.buffer[2]
        {
            error!(
                "JEDEC ID mismatch: expected {:02X} {:02X} {:02X}, got {:02X} {:02X} {:02X}",
                self.flash_info.manufacturer_id,
                self.flash_info.type_id,
                self.flash_info.capacity_id,
                self.buffer[0],
                self.buffer[1],
                self.buffer[2]
            );
            return Err(Error);
        }

        // reset()

        if self.write_state(true, 0x00).is_err() {
            return Err(Error);
        }

        if self.flash_info.size > (1 << 24) {
            if self.set_4byte_address_mode(true).is_err() {
                return Err(Error);
            }
        }

        self.init_ok = true;

        Ok(())
    }

    // fn reset(&self){
    //     let cmd = [cmd::MODE_CMD::MODE_RESET as u8];
    // }

    fn read_jedec_id(&mut self) -> Result<(), Error> {
        // Read JEDEC ID
        let cmd = [define::ID_CMD::JEDEC_ID as u8];
        if self
            .interface
            .write_and_read(&cmd, &mut self.buffer[..3])
            .is_ok()
        {
            info!(
                "JEDEC ID: {:02X} {:02X} {:02X}",
                self.buffer[0], self.buffer[1], self.buffer[2]
            );
            Ok(())
        } else {
            error!("Failed to read JEDEC ID");
            Err(Error)
        }
    }

    fn write_enable(&mut self, enable: bool) -> Result<(), Error> {
        // Write Enable
        let cmd = if enable {
            [define::WRITE_CMD::WRITE_ENABLE as u8]
        } else {
            [define::WRITE_CMD::WRITE_DISABLE as u8]
        };

        if self.interface.write(&cmd).is_err() {
            error!("Failed to write enable");
            return Err(Error);
        }

        if let Ok(status) = self.read_status() {
            if enable && (status & define::STATUS::WEL as u8) == 0 {
                error!("Write enable failed");
                Err(Error)
            } else if !enable && (status & define::STATUS::WEL as u8) != 0 {
                error!("Write disable failed");
                Err(Error)
            } else {
                Ok(())
            }
        } else {
            error!("Failed to read status register after write enable");
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
        let _ = self.write_enable(true);
        return ret;
    }

    fn wait_busy(&mut self) -> Result<(), Error> {
        // Wait for the flash to be ready
        for _ in 0..10 {
            if let Ok(status) = self.read_status() {
                if (status & define::STATUS::BUSY as u8) == 0 {
                    break;
                } else {
                    self.interface.delay(1);
                }
            } else {
                return Err(Error);
            }
        }
        Ok(())
    }

    fn set_4byte_address_mode(&mut self, enable: bool) -> Result<(), Error> {
        // Set 4-byte address mode
        if self.addr_4byte {
            return Ok(());
        }

        if self.write_enable(true).is_ok() {
            let cmd = if enable { [0xB7] } else { [0xE9] };
            if self.interface.write(&cmd).is_ok() {
                self.addr_4byte = enable;
            }
        }

        self.write_enable(false)
    }

    fn make_address_byte_array(&self, address: u32, buff: &mut [u8]) {
        let len = if self.addr_4byte { 4 } else { 3 };
        for i in 0..len {
            buff[i] = (address >> ((len - (i + 1)) * 8)) as u8;
        }
    }
}

macro_rules! check_init {
    ($self:ident) => {
        if !$self.init_ok {
            error!("Flash not initialized");
            panic!();
        }
    };
}

impl<I, const SIZE: usize> FlashOperations for Flash<I, SIZE>
where
    I: SerialInterface,
{
    fn erase_chip(&mut self) -> Result<(), Error> {
        check_init!(self);
        self.write_operation(|s|{
            let cmd = [define::ERASE_CMD::CHIP as u8];
            s.interface.write(&cmd)
        })
    }

    fn erase(&mut self, address: u32, size: usize) -> Result<(), Error> {
        check_init!(self);
        assert!(
            size != self.flash_info.secter_size as usize,
            "erase_size must be secter_size"
        );
        assert!(
            address % self.flash_info.secter_size != 0,
            "address must be secter_size aligned"
        );

        if (address + size as u32) > self.flash_info.size as u32 {
            return Err(Error);
        }
        if address == 0 && size == self.flash_info.size as usize {
            return self.erase_chip();
        }

        self.write_operation(|s| {
            let mut size = size;
            let mut cmd = [define::ERASE_CMD::BLOCK_64K as u8, 0, 0, 0, 0];
            let mut addr = address;
            while size > 0 {
                s.make_address_byte_array(addr, &mut cmd[1..]);
                let cmd_len = if s.addr_4byte { 5 } else { 4 };
                if s.interface.write(&cmd[..cmd_len]).is_err() {
                    error!("Failed to erase block at address {:08X}", addr);
                    return Err(Error);
                }
                if s.wait_busy().is_err(){
                    error!("Failed to wait for erase operation to complete");
                    return Err(Error);
                }
                if (addr % s.flash_info.secter_size as u32) != 0 {
                    if size > s.flash_info.secter_size as usize - (addr % s.flash_info.secter_size as u32) as usize {
                        size -= s.flash_info.secter_size as usize - (addr % s.flash_info.secter_size as u32) as usize;
                        addr += s.flash_info.secter_size as u32 - (addr % s.flash_info.secter_size as u32) as u32;
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

    fn erase_write(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        check_init!(self);

        todo!()
    }

    fn write(&mut self, address: u32, data: &[u8]) -> Result<(), Error> {
        check_init!(self);

        todo!()
    }

    fn read_data(&mut self, address: u32, buffer: &mut [u8]) -> Result<(), Error> {
        check_init!(self);

        todo!()
    }

    fn read_status(&mut self) -> Result<u8, Error> {
        check_init!(self);

        let cmd = [define::READ_CMD::STATUS_1 as u8];
        if self
            .interface
            .write_and_read(&cmd, &mut self.buffer[..1])
            .is_ok()
        {
            Ok(self.buffer[0])
        } else {
            error!("Failed to read status register");
            Err(Error)
        }
    }

    fn write_state(&mut self, is_volatile: bool, state: u8) -> Result<(), Error> {
        check_init!(self);

        // if is_volatile {
        //     //TODO: cmd 0x50 SFUD_VOLATILE_SR_WRITE_ENABLE
        //     let cmd = [0x50 as u8];
        //     if self.interface.write(&cmd).is_err() {
        //         return Err(Error);
        //     }
        // } else {
        if self.write_enable(true).is_err() {
            return Err(Error);
        };
        // }

        let cmd = [define::WRITE_CMD::WRIET_STATUS as u8, state];
        if self.interface.write(&cmd).is_err() {
            error!("Failed to write status register");
            return Err(Error);
        }
        Ok(())
    }
}
