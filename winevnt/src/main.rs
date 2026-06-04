use std::thread;
use std::time::Duration;

use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::{TraceTrait, UserTrace};
use ferrisetw::EventRecord;

use tracelogging as tlg;

// ------------------------------
// 1) 定义一个自定义 ETW Provider
// ------------------------------
tlg::define_provider!(MY_PROVIDER, "Demo.Bluetooth.Test");

// ------------------------------
// 2) 收到事件后的回调：打印到 console
// ------------------------------
fn on_event(record: &EventRecord, schema_locator: &SchemaLocator) {
    match schema_locator.event_schema(record) {
        Ok(schema) => {
            let provider_name = schema.provider_name();
            let event_id = record.event_id();

            print!("[recv] provider={} event_id={}", provider_name, event_id);

            // 只对我们自己发的 provider 尝试解析字段
            if provider_name == "Demo.Bluetooth.Test" {
                let parser = Parser::create(record, &schema);

                let seq: Result<u32, _> = parser.try_parse("Seq");
                let status: Result<u32, _> = parser.try_parse("Status");
                let device_name: Result<String, _> = parser.try_parse("DeviceName");

                if let Ok(v) = seq {
                    print!(" Seq={}", v);
                }
                if let Ok(v) = status {
                    print!(" Status={}", v);
                }
                if let Ok(v) = device_name {
                    print!(" DeviceName={}", v);
                }
            }

            println!();
        }
        Err(err) => {
            eprintln!(
                "[recv] schema parse failed, event_id={}, err={:?}",
                record.event_id(),
                err
            );
        }
    }
}

fn main() {
    // ------------------------------
    // 3) 注册 provider（发送端）
    // ------------------------------
    unsafe {
        MY_PROVIDER.register();
    }

    println!("provider registered: Demo.Bluetooth.Test");

    // ------------------------------
    // 4) 按 provider 名称创建监听器（监控端）
    // ------------------------------
    // 注意：先 register，再 by_name 查找，成功率更高
    let provider = Provider::by_name("Demo.Bluetooth.Test")
        .expect("Provider::by_name failed")
        .any(u64::MAX) // 接收所有 keyword
        .level(5)      // Verbose
        .add_callback(on_event)
        .build();

    // ------------------------------
    // 5) 启动实时 trace
    // ------------------------------
    let mut trace = UserTrace::new()
        .named("RustEtwSelfTest".to_string())
        .enable(provider)
        .start_and_process()
        .expect("start trace failed");

    println!("trace started, begin sending events...");

    // 让后台 trace 线程先稳定一下
    thread::sleep(Duration::from_millis(500));

    // ------------------------------
    // 6) 发送几条测试事件
    // ------------------------------
    for i in 0..5u32 {
        let status: u32 = if i % 2 == 0 { 1 } else { 0 };
        let device_name = format!("FakeBT-{:02}", i);

        tlg::write_event!(
            MY_PROVIDER,
            "BluetoothTestEvent",
            level(Verbose),
            keyword(0x1),
            u32("Seq", &i),
            u32("Status", &status),
            str8("DeviceName", device_name.as_str())
        );

        println!(
            "[send] Seq={} Status={} DeviceName={}",
            i, status, device_name
        );

        thread::sleep(Duration::from_millis(500));
    }

    // 给 monitor 一点时间把尾部事件处理完
    thread::sleep(Duration::from_secs(2));

    // ------------------------------
    // 7) 停止 trace，注销 provider
    // ------------------------------
    trace.stop();
    MY_PROVIDER.unregister();

    println!("done.");
}
