#![no_std]
#![no_main]

use panic_halt as _;
use rtt_target::{rprintln, rtt_init, rtt_init_print};

use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::gpio;
use embassy_stm32::rcc::*;
use embassy_time::{Duration, Timer};

use core::pin::Pin;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::Context;
use core::task::Poll;

static TICKS: AtomicU32 = AtomicU32::new(0);

// #[derive(Clone, Copy)]
struct CountFuture;

impl Future for CountFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Not allowed here due to Pin
        // let _moved = *self;

        let x = TICKS.fetch_add(1, Ordering::SeqCst);
        if (x % 1_000_000) == 0 {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[embassy_executor::task]
async fn task_1() {
    loop {
        let count = CountFuture;
        count.await;
        rprintln!("[task_1] wait count {}", TICKS.load(Ordering::Relaxed));
    }
}

#[embassy_executor::task]
async fn task_sh(
    mut tx: rtt_target::UpChannel,
    mut rx: rtt_target::DownChannel,)
{

    tx.write(b"\r\nSTM32 Embassy Shell\r\n");

    let mut buf = [0u8; 64];

    loop {
        tx.write(b"\r\nshell> ");

        let len = loop {
            let n = rx.read(&mut buf);

            if n > 0 {
                break n;
            }


            Timer::after(Duration::from_millis(10)).await;
        };

        command(&mut tx, &buf[..len]);
    }
}

fn command(tx: &mut rtt_target::UpChannel, cmd: &[u8]) {
    // remove CR/LF
    let cmd = trim(cmd);

    if cmd == b"help" {
        tx.write(
            b"\r\nCommands:\r\n\
              help        show commands\r\n\
              info        firmware info\r\n\
              hw          hardware info\r\n\
              led on      LED ON\r\n\
              led off     LED OFF\r\n",
        );
    } else if cmd == b"info" {
        tx.write(
            b"\r\nFirmware: Embassy STM32F103\r\n\
              Version: 0.1\r\n",
        );
    } else if cmd == b"hw" {
        tx.write(
            b"\r\nHardware:\r\n\
              MCU: STM32F103RB\r\n\
              Clock: 72MHz\r\n\
              RTOS: Embassy\r\n",
        );
    } else if cmd == b"led on" {
        tx.write(b"\r\nLED ON\r\n");

        // TODO:
        // led.set_high();
    } else if cmd == b"led off" {
        tx.write(b"\r\nLED OFF\r\n");

        // TODO:
        // led.set_low();
    } else {
        tx.write(b"\r\nUnknown command\r\n");
        tx.write(b"Type 'help'\r\n");
    }
}

fn trim(data: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = data.len();

    while start < end && (data[start] == b'\r' || data[start] == b'\n' || data[start] == b' ') {
        start += 1;
    }

    while end > start && (data[end - 1] == b'\r' || data[end - 1] == b'\n' || data[end - 1] == b' ')
    {
        end -= 1;
    }

    &data[start..end]
}

#[embassy_executor::task]
async fn task_led(mut led: gpio::Output<'static>, mut tx: rtt_target::UpChannel) {
    loop {
        // LED On
        led.set_low();
        tx.write(b"(task_led) low\n");
        Timer::after_millis(5000).await;

        // LED Off
        led.set_high();
        tx.write(b"(task_led) high\n");
        Timer::after_millis(5000).await;
    }
}

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
    spawner.spawn(task_sh( channels.up.0,
                           channels.down.0).unwrap());
    spawner.spawn(task_led(led, channels.up.1).unwrap());
}
