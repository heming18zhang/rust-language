use core::sync::atomic::Ordering;
use embassy_time::{Duration, Timer};

use crate::state::{
    LED_MODE,
    LedMode,
};

#[embassy_executor::task]
pub async fn task_sh(mut tx: rtt_target::UpChannel, mut rx: rtt_target::DownChannel) {
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
              led off     LED OFF\r\n\
              led         LED current status\r\n",
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

        LED_MODE.store(LedMode::On as u8, Ordering::Relaxed);
    } else if cmd == b"led off" {
        tx.write(b"\r\nLED OFF\r\n");

        LED_MODE.store(LedMode::Off as u8, Ordering::Relaxed);
    } else if cmd == b"led blink" {
        tx.write(b"\r\nLED BLINK\r\n");

        LED_MODE.store(LedMode::Blink as u8, Ordering::Relaxed);
    } else if cmd == b"led" {
        tx.write(b"\r\nLED query\r\n");
        let mode = LedMode::from_u8(LED_MODE.load(Ordering::Relaxed));

        match mode {
            LedMode::Off => {
                tx.write(b"LED state is OFF\r\n");
            }

            LedMode::On => {
                tx.write(b"LED state is ON\r\n");
            }

            LedMode::Blink => {
                tx.write(b"LED state is BLINK\r\n");
            }
        }
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

