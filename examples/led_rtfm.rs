//! A complete working example.
//!
//! This requires `cortex-m-rtfm` v0.5.
//!
//! It uses `TIMER1` to drive the display, and `RTC0` to update a simple
//! animated image.
#![no_main]
#![no_std]

use panic_halt as _;

use microbit::{
    display::{self, image::GreyscaleImage, Display, Frame, MicrobitDisplayTimer, MicrobitFrame},
    hal::rtc::{Rtc, RtcInterrupt},
    pac,
};
use rtic::app;

fn heart_image(inner_brightness: u8) -> GreyscaleImage {
    let b = inner_brightness;
    GreyscaleImage::new(&[
        [0, 7, 0, 7, 0],
        [7, b, 7, b, 7],
        [7, b, b, b, 7],
        [0, 7, b, 7, 0],
        [0, 0, 7, 0, 0],
    ])
}

#[app(device = microbit::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        gpio: pac::GPIO,
        display_timer: MicrobitDisplayTimer<pac::TIMER1>,
        anim_timer: Rtc<pac::RTC0>,
        display: Display<MicrobitFrame>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        let mut p: pac::Peripherals = cx.device;

        // Starting the low-frequency clock (needed for RTC to work)
        p.CLOCK.tasks_lfclkstart.write(|w| unsafe { w.bits(1) });
        while p.CLOCK.events_lfclkstarted.read().bits() == 0 {}
        p.CLOCK.events_lfclkstarted.reset();

        // RTC at 16Hz (32_768 / (2047 + 1))
        // 16Hz; 62.5ms period
        let mut rtc0 = Rtc::new(p.RTC0, 2047).unwrap();
        rtc0.enable_event(RtcInterrupt::Tick);
        rtc0.enable_interrupt(RtcInterrupt::Tick, None);
        rtc0.enable_counter();

        let mut timer = MicrobitDisplayTimer::new(p.TIMER1);
        display::initialise_display(&mut timer, &mut p.GPIO);

        init::LateResources {
            gpio: p.GPIO,
            display_timer: timer,
            anim_timer: rtc0,
            display: Display::new(),
        }
    }

    #[task(binds = TIMER1, priority = 2,
           resources = [display_timer, gpio, display])]
    fn timer1(mut cx: timer1::Context) {
        display::handle_display_event(
            &mut cx.resources.display,
            cx.resources.display_timer,
            cx.resources.gpio,
        );
    }

    #[task(binds = RTC0, priority = 1,
           resources = [anim_timer, display])]
    fn rtc0(mut cx: rtc0::Context) {
        static mut FRAME: MicrobitFrame = MicrobitFrame::const_default();
        static mut STEP: u8 = 0;

        cx.resources.anim_timer.reset_event(RtcInterrupt::Tick);

        let inner_brightness = match *STEP {
            0..=8 => 9 - *STEP,
            9..=12 => 0,
            _ => unreachable!(),
        };

        FRAME.set(&heart_image(inner_brightness));
        cx.resources.display.lock(|display| {
            display.set_frame(FRAME);
        });

        *STEP += 1;
        if *STEP == 13 {
            *STEP = 0
        };
    }
};
