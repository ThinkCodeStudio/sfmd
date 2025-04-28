
pub(crate) enum WriteCmd {
    WriteEnable = 0x06,
    WriteDisable = 0x04,
    WrietStatus = 0x01,
    PageProgram = 0x02,
}

pub(crate) enum ReadCmd {
    Status1 = 0x05,
    Status2 = 0x35,
    Data = 0x03,
    Fast = 0x0B,
    FastDual = 0x3B,
    FastDualIo = 0xBB,
}
pub(crate) enum ModeCmd {
    PowerDown = 0xB9,
    HighPerformance = 0xA3,
    ModeReset = 0xFF,
    ReleasePowerDown = 0xAB,
}

pub(crate) enum IdCmd {
    DeviceId = 0xAB,
    Manufacturer = 0x90,
    ReadUnique = 0x4B,
    JedecId = 0x9F,
}

pub(crate) enum EraseCmd {
    Block64k = 0xD8,
    Block32k = 0x52,
    Sector4k = 0x20,
    Chip = 0xC7, // C7h|60h
    Suspend = 0x75,
    Resume = 0x7A,
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