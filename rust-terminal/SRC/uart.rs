use serialport::SerialPort;
use std::io::{self, Read, Write};
use std::time::Duration;

/// UART 终端封装
pub struct UartTerminal {
    port: Box<dyn SerialPort>,
}

impl UartTerminal {
    /// 打开串口
    pub fn open(port_name: &str, baud: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let port = serialport::new(port_name, baud)
            .timeout(Duration::from_millis(10))
            .open()?;

        Ok(Self { port })
    }

    /// 发送数据
    pub fn send(&mut self, data: &[u8]) -> Result<(), io::Error> {
        self.port.write_all(data)?;
        self.port.flush()?;
        Ok(())
    }

    /// 接收数据
    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.port.read(buf)
    }
}

/// 判断是否 ASCII 可打印
pub fn is_ascii_printable(data: &[u8]) -> bool {
    data.iter().all(|b| {
        b.is_ascii_graphic()
            || *b == b' '
            || *b == b'\n'
            || *b == b'\r'
            || *b == b'\t'
    })
}

/// HEX 打印（8 bytes per line）
pub fn print_hex(data: &[u8]) {
    for chunk in data.chunks(8) {
        for b in chunk {
            print!("{:02X} ", b);
        }
        println!();
    }
}
