pub(crate) enum WRITE_CMD {
    WRITE_ENABLE = 0x06,
    WRITE_DISABLE = 0x04,
    WRIET_STATUS = 0x01,
    PAGE_PROGRAM = 0x02,
}

pub(crate) enum READ_CMD {
    STATUS_1 = 0x05,
    STATUS_2 = 0x35,
    DATA = 0x03,
    FAST = 0x0B,
    FAST_DUAL = 0x3B,
    FAST_DUAL_IO = 0xBB,
}
pub(crate) enum MODE_CMD {
    POWER_DOWN = 0xB9,
    HIGH_PERFORMANCE = 0xA3,
    MODE_RESET = 0xFF,
    RELEASE_POWER_DOWN = 0xAB,
}

pub(crate) enum ID_CMD {
    DEVICE_ID = 0xAB,
    MANUFACTURER = 0x90,
    READ_UNIQUE = 0x4B,
    JEDEC_ID = 0x9F,
}

pub(crate) enum ERASE_CMD {
    BLOCK_64K = 0xD8,
    BLOCK_32K = 0x52,
    SECTOR_4K = 0x20,
    CHIP = 0xC7, // C7h|60h
    SUSPEND = 0x75,
    RESUME = 0x7A,
}

pub(crate) enum STATUS {
    BUSY = 0b0000_0000_0001,
    WEL = 0b0000_0000_0010,
    BP0 = 0b0000_0000_0100,
    BP1 = 0b0000_0000_1000,
    BP2 = 0b0000_0001_0000,
    TB = 0b0000_0010_0000,
    SEC = 0b0000_0100_0000,
    SRP0 = 0b0000_1000_0000,
    SRP1 = 0b0001_0000_0000,
    QE = 0b0010_0000_0000,
}

#[cfg(feature = "qspi")]
pub(crate) enum QUAD_CMD {
    PAGE_PROGRAM = 0x32,
    FAST_DUAL = 0x6B,
    FAST_DUAL_IO = 0xEB,
}