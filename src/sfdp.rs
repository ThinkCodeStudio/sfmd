use core::fmt::Error;
use crate::serial_interface::SerialInterface;

struct SFDPInfo {
    manufacturer_id: u8,
    type_id: u8,
    capacity_id: u8,
    size: usize,
    secter_size: u32,
}

struct SFDP<I>
where
    I: SerialInterface,
{
    cmd: [u8; 5],
    interface: I,
}

impl<I> SFDP<I>
where
    I: SerialInterface,
{

    fn read_sfdp_data(
        &mut self,
        interface: &mut I,
        address: u32,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        // Read SFDP data from the specified address
        // self.cmd[0] = 0x5A; // SFDP command
        self.cmd[1] = (address >> 16) as u8;
        self.cmd[2] = (address >> 8) as u8;
        self.cmd[3] = address as u8;
        // self.cmd[4] = 0xFF; // Dummy byte

        interface.write_and_read(&self.cmd, buffer)
    }

    pub fn new(interface: I) -> Self {
        SFDP {
            cmd: [0x5A, 0, 0, 0, 0xff],
            interface: interface,
        }
    }

    // pub fn create(&self) -> Option<SFDPInfo> {
    //     // Read SFDP header
    //     let mut header = [0; 4];
    //     self.interface.read(&mut header, 4).ok()?;

    //     // Check if the header is valid
    //     if header[0] != 0x53 || header[1] != 0x46 || header[2] != 0x44 || header[3] != 0x50 {
    //         return None;
    //     }

    //     // Read the rest of the SFDP data
    //     // ...

    //     Some(sfdp_info)
    // }
}

