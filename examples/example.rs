#![no_std]
#![no_main]

use core::{cell::{Cell, RefCell}, fmt::Error};
// pick a panicking behavior
use panic_rtt_target as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use log::{Level, LevelFilter, Metadata, Record, debug, info};
use rtt_target::{rprintln, rtt_init_print};
use sfmd_rs::{FlashInfo, flash, serial_interface::SerialInterface};
use stm32f4xx_hal::{
    gpio::{Output, Pin, PushPull, Speed},
    hal::spi,
    pac,
    prelude::*,
    rcc::RccExt,
    spi::*,
    timer::SysDelay,
};

use sfmd_rs::flash::Flash;

pub struct Logger {
    level: Level,
}

static LOGGER: Logger = Logger {
    level: Level::Debug,
};

pub fn log_init() {
    rtt_init_print!();
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        rprintln!("{} - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

struct SpiDev<'a, SPI, const G: char, const P: u8>
where
    SPI: Instance,
{
    spi: Spi<SPI>,
    cs: Pin<G, P, Output<PushPull>>,
    delay: &'a Cell<SysDelay>,
}

impl<'a, SPI, const G: char, const P: u8> SpiDev<'a, SPI, G, P>
where
    SPI: Instance,
{
    pub fn new(spi: Spi<SPI>, cs: Pin<G, P, Output<PushPull>>, delay: &'a Cell<SysDelay>) -> Self {
        SpiDev { spi, cs, delay }
    }
}

impl<'a, SPI, const G: char, const P: u8> SerialInterface for SpiDev<'a, SPI, G, P>
where
    SPI: Instance,
{
    fn write(&mut self, cmd: &[u8]) -> Result<(), core::fmt::Error> {
        self.cs.set_low();
        self.spi.write(cmd).unwrap();
        self.cs.set_high();
        Ok(())
    }

    fn write_and_read(&mut self, cmd: &[u8], rev: &mut [u8]) -> Result<(), Error> {
        self.cs.set_low();
        self.spi.write(cmd).unwrap();
        self.spi.read(rev).unwrap();
        self.cs.set_high();
        Ok(())
    }

    fn delay(&mut self, ms: u32) {
        self.delay.borrow_mut().delay(ms.millis()); // Assuming 168 MHz clock speed
    }
}

#[entry]
fn main() -> ! {
    log_init();
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    info!("Starting up...");
    let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(168.MHz()).freeze();

    let iog = dp.GPIOG.split();
    let iob = dp.GPIOB.split();

    let clk = iob.pb3.into_alternate::<5>().speed(Speed::VeryHigh);
    let miso = iob.pb4.into_alternate::<5>().speed(Speed::VeryHigh);
    let mosi = iob
        .pb5
        .into_alternate::<5>()
        .speed(Speed::VeryHigh)
        .internal_pull_up(true);
    let mut cs = iog.pg3.into_push_pull_output();
    cs.set_high();

    let mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let spi1 = Spi::new(dp.SPI1, (clk, miso, mosi), mode, 5.MHz(), &clocks);

    let delay = Cell::new(cp.SYST.delay(&clocks));


    let spi_device = SpiDev::new(spi1, cs, delay);
    type SpiDeviceType<'a> = SpiDev<pac::SPI1, 'G', 3>;

    let flash_info = FlashInfo::new(0xEF, 0x40, 0x17, 16 * 1024 * 1024, 4096);

    let mut flash = Flash::<SpiDeviceType, 4096>::new(spi_device, flash_info);

    info!("init flash...");
    flash.init().unwrap();
    info!("init flash done.");
    loop {
        // Your application logic goes here
        loop_delay.get_mut().delay(1.secs());
    }
}
