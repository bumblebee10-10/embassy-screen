use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig;
use embassy_rp::peripherals::*;
use embassy_rp::pwm::Pwm;
use embassy_rp::spi::Spi;
use embassy_time::{Delay, Duration, Timer};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor, Size},
    primitives::{PrimitiveStyleBuilder, Rectangle},
    Drawable,
};

use crate::idisplay::SPIDeviceInterface;

pub(crate) const SCREEN_X_PX: u16 = 320u16;
pub(crate) const SCREEN_Y_PX: u16 = 240u16;

pub static FRAMES: Channel<CriticalSectionRawMutex, crate::UIScene, 25> = Channel::new();

pub struct NewScreen {
    pub backlight: Option<Pwm<'static, PWM_CH0>>,
}

impl NewScreen {
    pub fn new() -> Self {
        Self { backlight: None }
    }

    pub async fn run(
        &mut self,
        di: SPIDeviceInterface<
            SpiDeviceWithConfig<
                'static,
                CriticalSectionRawMutex,
                Spi<'static, SPI0, embassy_rp::spi::Async>,
                Output<'static, PIN_5>,
            >,
            Output<'static, PIN_7>,
        >,
        rst: PIN_6,
    ) {
        let rst = Output::new(rst, Level::High);

        use st7789::Orientation;
        let mut display: st7789::ST7789<
            SPIDeviceInterface<
                SpiDeviceWithConfig<
                    CriticalSectionRawMutex,
                    Spi<'static, SPI0, embassy_rp::spi::Async>,
                    Output<'static, PIN_5>,
                >,
                Output<'static, PIN_7>,
            >,
            Output<'static, PIN_6>,
        > = st7789::ST7789::new(di, rst, SCREEN_Y_PX, SCREEN_X_PX);

        display.init(&mut Delay).unwrap();
        display.set_orientation(Orientation::Landscape).unwrap();

        loop {
            match FRAMES.try_recv() {
                Ok(scene) => match scene {
                    crate::UIScene::A => do_a(&mut display),
                    crate::UIScene::B => do_b(&mut display),
                },
                Err(_) => (),
            }
            Timer::after(Duration::from_millis(40)).await;
        }
    }
}

pub type DisplayT = st7789::ST7789<
    SPIDeviceInterface<
        SpiDeviceWithConfig<
            'static,
            CriticalSectionRawMutex,
            Spi<'static, embassy_rp::peripherals::SPI0, embassy_rp::spi::Async>,
            Output<'static, embassy_rp::peripherals::PIN_5>,
        >,
        Output<'static, embassy_rp::peripherals::PIN_7>,
    >,
    Output<'static, embassy_rp::peripherals::PIN_6>,
>;

pub fn do_a(display: &mut DisplayT) {
    let s = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .build();
    let a = Rectangle::new(
        Point::new(0, 0),
        Size::new(SCREEN_X_PX as u32, SCREEN_Y_PX as u32),
    )
    .into_styled(s);
    a.draw(display).unwrap();
}

pub fn do_b(display: &mut DisplayT) {
    let s = embedded_graphics::primitives::PrimitiveStyleBuilder::new()
        .fill_color(embedded_graphics::pixelcolor::Rgb565::RED)
        .build();
    let b = Rectangle::new(
        Point::new(0, 0),
        Size::new(SCREEN_X_PX as u32, SCREEN_Y_PX as u32),
    )
    .into_styled(s);
    b.draw(display).unwrap();
}

pub fn do_c(display: &mut DisplayT) {
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::RED)
        .stroke_width(3)
        .fill_color(Rgb565::GREEN)
        .build();

    Rectangle::new(Point::new(30, 20), Size::new(10, 15))
        .into_styled(style)
        .draw(display)
        .unwrap();
}
