#![no_std]
#![no_main]

use core::{cell::{Cell, RefCell}, error, fmt::Error};
use cortex_m::interrupt::Mutex;
// pick a panicking behavior
use panic_rtt_target as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// use panic_abort as _; // requires nightly
// use panic_itm as _; // logs messages over ITM; requires ITM support
// use panic_semihosting as _; // logs messages to the host stderr; requires a debugger

use cortex_m_rt::entry;

use log::{debug, error, info, Level, LevelFilter, Metadata, Record};
use rtt_target::{rprintln, rtt_init_print};
use sfmd_rs::{flash, serial_interface::SerialInterface, FlashInfo, FlashOperations};
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
    delay:&'a mut SysDelay,
}

impl<'a, SPI, const G: char, const P: u8> SpiDev<'a, SPI, G, P>
where
    SPI: Instance,
{
    pub fn new(spi: Spi<SPI>, cs: Pin<G, P, Output<PushPull>>, delay:&'a mut SysDelay) -> Self {
        SpiDev { spi, cs, delay }
    }
}

impl<'a, SPI, const G: char, const P: u8> SerialInterface for SpiDev<'a, SPI, G, P>
where
    SPI: Instance,
{
    fn write(&mut self, cmd: &[u8], data:Option<&[u8]>) -> Result<(), core::fmt::Error> {
        self.cs.set_low();
        self.spi.write(cmd).unwrap();
        if let Some(data) = data {
            self.spi.write(data).unwrap();
        }
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
        self.delay.delay(ms.millis()); // Assuming 168 MHz clock speed
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

    let mut spi_delay = cp.SYST.delay(&clocks);


    let spi_device = SpiDev::new(spi1, cs, &mut spi_delay);

    let flash_info = FlashInfo::new(0xEF, 0x40, 0x17, 16 * 1024 * 1024, 4096);
    
    
    const TEST_DATA_SIZE: usize = 4000;
    let test_data = [0xAB_u8; TEST_DATA_SIZE];
    let mut buffer = [0_u8; TEST_DATA_SIZE];
    
    info!("init flash...");
    if let Ok(flash) =&mut Flash::new(spi_device, flash_info){
        info!("init flash done.");
        // flash.erase(0, 4096).unwrap(); // ok
        flash.write_data(0, &test_data).unwrap(); // ok
        flash.read_data(0, &mut buffer).unwrap(); // ok
        if test_data == buffer {
            info!("flash read and write ok.");
        } else {
            error!("flash read and write failed. read data: {:?}, write data: {:?}", &buffer[..8], &test_data[..8]);
        }
    }
    else{
        error!("flash init failed.");   
    }

    loop {
        spi_delay.delay(1.secs());
    }
}
