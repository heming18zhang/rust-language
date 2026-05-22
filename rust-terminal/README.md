# UART Terminal (Rust)

Cross-platform UART terminal written in Rust.

Supports:

- Linux
- Windows
- macOS

Features:

- UART 115200bps
- Keyboard -> UART TX
- UART RX -> terminal
- ASCII display
- Non-ASCII display in HEX
- 8 bytes per HEX line
- Multi-threaded RX/TX

---

# Build

Install Rust:

https://www.rust-lang.org/tools/install

Build:

```bash
cargo build --release


Usage:
       Windows: uart_terminal COM3
       Linux  : uart_terminal /dev/ttyUSB0
       macOS  : uart_terminal /dev/tty.usbserial

Examples:

# ===============================
# UART Terminal <-> Zephyr OS Log
# ===============================

Opening UART: /dev/ttyUSB0

*** Booting Zephyr OS build v3.6.0 ***

[00:00:00.000,000] <inf> os: Main thread started
[00:00:00.000,100] <inf> os: CPU: Cortex-M4
[00:00:00.000,200] <inf> os: Booting Zephyr
[00:00:00.000,300] <inf> kernel: scheduling started

uart:~$ help
Available commands:
  clear
  echo
  kernel
  device
  uart
  sensor

uart:~$ kernel version
Zephyr version 3.6.0

uart:~$ echo hello zephyr
echo: hello zephyr

uart:~$ gpio set led0
LED0 ON

uart:~$ sensor get temp
temp: 25.34 C

uart:~$ uart dump

[HEX]
AA 01 02 03 04 05 06 07
08 09 0A 0B 0C 0D 0E 0F
FF 10 20 30 40 50 60 70
......
