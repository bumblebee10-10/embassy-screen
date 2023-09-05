#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::cell::RefCell;

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, SPI0};

use defmt_rtt as _;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

mod display;
mod idisplay;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    spawner
        .spawn(run(
            p.SPI0, p.PIN_4, p.PIN_3, p.PIN_2, p.DMA_CH0, p.DMA_CH1, p.PIN_6, p.PIN_7, p.PIN_5,
        ))
        .unwrap();

    let async_input = Input::new(p.PIN_16, Pull::None);

    let delay = Duration::from_secs(1);
    loop {
        // info!("drawing A!");
        draw_a().await;
        async_input.wait_for_high().await;

        Timer::after(delay).await;

        // info!("drawing B!");
        draw_b().await;
        async_input.wait_for_low().await;
    }
}

use static_cell::StaticCell;

static SPI_BUS: StaticCell<
    Mutex<CriticalSectionRawMutex, RefCell<embassy_rp::spi::Spi<SPI0, embassy_rp::spi::Async>>>,
> = StaticCell::new();

#[embassy_executor::task]
pub async fn run(
    dev: SPI0,
    miso: PIN_4,
    mosi: PIN_3,
    clk: PIN_2,
    dma_ch0: DMA_CH0,
    dma_ch1: DMA_CH1,
    rst: PIN_6,
    dcx: PIN_7,
    cs: PIN_5,
) {
    let dcx = Output::new(dcx, Level::High);

    let mut display_config = embassy_rp::spi::Config::default();
    display_config.frequency = 62_500_000u32;
    display_config.phase = embassy_rp::spi::Phase::CaptureOnFirstTransition;
    display_config.polarity = embassy_rp::spi::Polarity::IdleLow;

    let spi = embassy_rp::spi::Spi::new(
        dev,
        clk,
        mosi,
        miso,
        dma_ch0,
        dma_ch1,
        embassy_rp::spi::Config::default(),
    );
    let rc = RefCell::new(spi);

    let spi_bus = SPI_BUS.init(Mutex::new(rc));

    let spi_w_cfg = SpiDeviceWithConfig::new(spi_bus, Output::new(cs, Level::High), display_config);
    let mut screen = display::NewScreen::new();
    let di = idisplay::SPIDeviceInterface::new(spi_w_cfg, dcx);

    screen.run(di, rst).await;
}

pub enum UIScene {
    A,
    B,
}
pub static FRAMES: Channel<CriticalSectionRawMutex, UIScene, 2> = Channel::new();
async fn draw_a() -> ! {
    loop {
        FRAMES.send(UIScene::A).await;
        Timer::after(Duration::from_millis(1000)).await;
    }
}
async fn draw_b() -> ! {
    loop {
        FRAMES.send(UIScene::B).await;
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
