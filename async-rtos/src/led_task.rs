use core::sync::atomic::Ordering;
use embassy_stm32::gpio;

use crate::state::{
    LED_MODE,
    LedMode,
};

use embassy_stm32::gpio::Output;
use embassy_time::Timer;

#[embassy_executor::task]
pub async fn task_led(mut led: gpio::Output<'static>, mut tx: rtt_target::UpChannel) {
    let mut last_mode = LedMode::Off;

    loop {
        let mode = LedMode::from_u8(LED_MODE.load(Ordering::Relaxed));

        if mode != last_mode {
            last_mode = mode;

            match mode {
                LedMode::Off => {
                    led.set_low();
                    tx.write(b"\r\nLED is off\r\n");
                }

                LedMode::On => {
                    led.set_high();
                    tx.write(b"\r\nLED is on\r\n");
                }

                LedMode::Blink => {
                    // handled below
                    tx.write(b"\r\nLED is blinking\r\n");
                }
            }
        }

        if mode == LedMode::Blink {
            led.toggle();
            embassy_time::Timer::after_millis(500).await;
        } else {
            embassy_time::Timer::after_millis(5).await;
        }
    }
}
