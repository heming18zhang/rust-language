#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::gpio::{Level, Output, OutputConfig};
use embassy_net::{Runner, StackResources};
use static_cell::StaticCell;
use esp_radio::wifi::{
        sta::StationConfig,
        AuthenticationMethodConfig,
        Config,
        Interface,
        WifiController,
    };
use esp_hal::Async;
use esp_hal::usb::usb_serial_jtag::UsbSerialJtag;
use embassy_shell::Shell;
use embassy_net::icmp::ping::{PingManager, PingParams};
use embassy_net::icmp::PacketMetadata;
use core::net::Ipv4Addr;
use core::str::FromStr;
use alloc::format;
use alloc::boxed::Box;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!{"in panic info:{}\r", info};
    loop {}
}

#[embassy_executor::task]
async fn net_task(
    mut runner: Runner<'static, Interface>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn shell_task(
    usb: UsbSerialJtag<'static, Async>,
    stack: embassy_net::Stack<'static>,
) {


    let (mut rx, mut tx) = usb.split();

    let mut shell = Shell::new();
    shell.prompt("> ");

    // ping <IPv4 address>
    shell.add_command("ping", "Ping an IPv4 address", move |args, mut io| {
        let target = args
            .get(0)
            .and_then(|s| Ipv4Addr::from_str(s).ok());

        let stack = stack;

        Box::pin(async move {
            let Some(target) = target else {
                return io.println("Usage: ping 192.168.1.1").await;
            };

            let mut rx_meta = [PacketMetadata::EMPTY];
            let mut tx_meta = [PacketMetadata::EMPTY];
            let mut rx_buffer = [0u8; 256];
            let mut tx_buffer = [0u8; 256];

            let mut manager = PingManager::new(
                stack,
                &mut rx_meta,
                &mut rx_buffer,
                &mut tx_meta,
                &mut tx_buffer,
            );

            for i in 1..=10 {
                let mut params = PingParams::new(target);
                params.set_count(1);

                match manager.ping(&params).await {
                    Ok(time) => {
                        io.println(&format!(
                            "Ping {}: {} ms",
                            i,
                            time.as_millis()
                        ))
                        .await;
                    }
                    Err(e) => {
                        io.println(&format!(
                            "Ping {} failed: {:?}",
                            i, e
                        ))
                        .await;
                    }
                }

                Timer::after(Duration::from_secs(1)).await;
            }
            Ok(())
        })
    });

    let _ = shell.run(&mut rx, &mut tx).await;
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.4.0
    // generator parameters: -o esp32c3 -o unstable-hal -o alloc -o wifi -o embassy -o esp32c3-mini-1
    esp_println::println!("1. Entered main");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_println::println!("=== ENTERED MAIN FUNCTION ===");
    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO2
    // - GPIO8
    // - GPIO9
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _gpio11 = peripherals.GPIO11;
    let _gpio12 = peripherals.GPIO12;
    let _gpio13 = peripherals.GPIO13;
    let _gpio14 = peripherals.GPIO14;
    let _gpio15 = peripherals.GPIO15;
    let _gpio16 = peripherals.GPIO16;
    let _gpio17 = peripherals.GPIO17;
    let mut led_blue = Output::new(
        peripherals.GPIO8,
        Level::High, // Active-low LED: initially OFF
        OutputConfig::default(),
    );

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);
    esp_println::println!("3. eps_rtos started");
    
    // ===========================================
    

    let mut wifi_controller =
        WifiController::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let wifi_interface = Interface::station();

    let station_config = StationConfig::default()
        .with_ssid("tst".try_into().unwrap())
        .with_authentication(
            AuthenticationMethodConfig::Wpa2Personal(
                "hztest18".try_into().unwrap(),
            ),
        );

    wifi_controller
        .set_config(&Config::Station(station_config))
        .expect("Failed to set Wi-Fi config");

    wifi_controller
        .connect_async()
        .await
        .expect("Failed to connect to AP");

    esp_println::println!("Wi-Fi connected successfully!");

    
    // ===========================================
    // 1. Configure DHCP
    let net_config = embassy_net::Config::dhcpv4(Default::default());

    // 2. Allocate network resources
    static RESOURCES: StaticCell<StackResources<3>> =
        StaticCell::new();

    let resources = RESOURCES.init(StackResources::new());

    // 3. Create the network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        resources,
        1234,
    );

    // 4. Start the network task
    spawner.spawn(net_task(runner).unwrap());

    // 5. Wait for DHCP
    esp_println::println!("Waiting for DHCP...");
    stack.wait_config_up().await;

    // 6. Print IP address
    if let Some(config) = stack.config_v4() {
        esp_println::println!("IP address: {}", config.address);
        esp_println::println!("Gateway: {:?}", config.gateway);
    }

    //================================
    // Start USB Serial/JTAG shell task
    let usb = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();
    spawner.spawn(shell_task(usb, stack).unwrap()); 
    //==================================


    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        //esp_println::println!("in looping.");
        led_blue.toggle();
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
