#![no_std]
#![no_main]

use panic_halt as _;
use rtt_target::{rprintln, rtt_init, rtt_init_print};

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::*;
use embassy_stm32::gpio;

use core::pin::Pin;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Context;
use core::task::Poll;

static TICKS: AtomicU32 = AtomicU32::new(0);
use a_rtos::{led_task::task_led, shell_task::task_sh};


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    //rtt_init_print!();
    let channels = rtt_init! {
        up: {
            0: {
                size: 512
            }
            1: {
                size: 128
                //abcname: "log"
            }
        }
        down: {
            0: {
                size: 128
            }
        }
    };
    let mut config = Config::default();
    //there's no HSE in F103RB board
    /*   config.rcc.hse = Some(Hse {
            freq: embassy_stm32::time::Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
    */
    config.rcc.pll = Some(Pll {
        src: PllSource::HSI,
        prediv: PllPreDiv::DIV2,
        mul: PllMul::MUL16,
    });

    config.rcc.sys = Sysclk::PLL1_P;

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV2;
    config.rcc.apb2_pre = APBPrescaler::DIV1;

    rprintln!("befoe init sysclk");
    let p = embassy_stm32::init(config);
    // let clocks = embassy_stm32::rcc::clocks();
    rprintln!("sysclk done {:?}"); //, clocks.sys);
    let led = gpio::Output::new(p.PA5, gpio::Level::High, gpio::Speed::Low);

    //spawner.spawn(task_1().unwrap());
    spawner.spawn(task_sh(channels.up.0, channels.down.0).unwrap());
    spawner.spawn(task_led(led, channels.up.1).unwrap());
}
