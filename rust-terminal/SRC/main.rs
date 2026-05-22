mod uart;

use uart::{UartTerminal, is_ascii_printable, print_hex};
use std::env;
use std::io::{self, Write};
use std::thread;

/// UART RX 线程
fn rx_loop(mut uart: UartTerminal) {
    let mut buf = [0u8; 4096];

    loop {
        match uart.recv(&mut buf) {
            Ok(n) if n > 0 => {
                let data = &buf[..n];

                if is_ascii_printable(data) {
                    if let Ok(s) = std::str::from_utf8(data) {
                        print!("{}", s);
                        io::stdout().flush().unwrap();
                    } else {
                        println!("\n[HEX]");
                        print_hex(data);
                    }
                } else {
                    println!("\n[HEX]");
                    print_hex(data);
                }
            }

            Ok(_) => {}

            Err(_) => {
                println!("Cannot open serial port. Check USB connector!");
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: uart_terminal <port>");
        eprintln!("Example: uart_terminal COM3");
        return Ok(());
    }

    let port = &args[1];

    println!("Opening UART: {}", port);

    // RX UART instance
    let uart_rx = UartTerminal::open(port, 115200)?;

    // TX UART instance
    let mut uart_tx = UartTerminal::open(port, 115200)?;

    // RX thread
    thread::spawn(move || {
        rx_loop(uart_rx);
    });

    // TX loop (keyboard -> UART)
    let stdin = io::stdin();

    loop {
        let mut input = String::new();

        stdin.read_line(&mut input)?;

        uart_tx.send(input.as_bytes())?;
    }
}
