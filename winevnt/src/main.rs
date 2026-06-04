use std::env;
use std::process::Command;
use std::thread;
use std::time::Duration;

use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::{TraceTrait, UserTrace};
use ferrisetw::EventRecord;

use tracelogging_dynamic as tld;

// ------------------------------------------------------------
// ETW command line tool
//
// Subcommands:
//
//   list
//     Run "logman query providers" and print all OS registered ETW providers.
//
//   monitor
//     Monitor one or more ETW providers in real time.
//
//   generate
//     Generate ETW events with runtime provider name,
//     runtime event name, and runtime fields.
//
// Build:
//
//   cargo build --release
//
// Examples:
//
//   .\target\release\etw_send_and_monitor.exe list
//
//   .\target\release\etw_send_and_monitor.exe monitor "Demo.Bluetooth.Test"
//
//   .\target\release\etw_send_and_monitor.exe monitor "{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}"
//
//   .\target\release\etw_send_and_monitor.exe generate "Demo.Bluetooth.Test" "ConnectEvent" "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"
//
//   .\target\release\etw_send_and_monitor.exe generate "Demo.Bluetooth.Test" "ConnectEvent" --count 5 --interval-ms 500 "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"
//
// Supported field syntax:
//
//   FieldName=string:value
//   FieldName=u32:value
//   FieldName=i64:value
//   FieldName=bool:true
//   FieldName=bool:false
// ------------------------------------------------------------

#[derive(Debug, Clone)]
enum DynamicField {
    Str(String, String),
    U32(String, u32),
    I64(String, i64),
    Bool(String, bool),
}

fn print_usage() {
    // Important:
    // Use eprintln!("{}", raw_string) instead of eprintln!(raw_string).
    // Otherwise "{guid}" inside the help text is parsed as a format placeholder.
    eprintln!(
        "{}",
        r#"Usage:
  etw_send_and_monitor.exe list

  etw_send_and_monitor.exe monitor <provider1> [provider2] [provider3] ...

  etw_send_and_monitor.exe generate <provider_name> <event_name> [--count N] [--interval-ms N] [fields...]

Examples:
  etw_send_and_monitor.exe list

  etw_send_and_monitor.exe monitor "Demo.Bluetooth.Test"

  etw_send_and_monitor.exe monitor "{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}"

  etw_send_and_monitor.exe generate "Demo.Bluetooth.Test" "ConnectEvent" "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"

  etw_send_and_monitor.exe generate "Demo.Bluetooth.Test" "ConnectEvent" --count 5 --interval-ms 500 "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"

Field format:
  Name=string:value
  Name=u32:value
  Name=i64:value
  Name=bool:true|false
"#
    );
}

fn run_list() {
    // Run the built-in Windows ETW provider listing command.
    // This is equivalent to running:
    //
    //   logman query providers
    //
    // The output includes OS registered ETW provider names and GUIDs.
    let output = Command::new("logman")
        .args(["query", "providers"])
        .output()
        .unwrap_or_else(|e| panic!("failed to run 'logman query providers': {}", e));

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        eprintln!(
            "[error] logman query providers failed with exit code: {:?}",
            output.status.code()
        );
    }
}

fn looks_like_guid(s: &str) -> bool {
    let trimmed = s.trim_matches(|c| c == '{' || c == '}');

    if trimmed.len() != 36 {
        return false;
    }

    for (i, ch) in trimmed.as_bytes().iter().enumerate() {
        if [8, 13, 18, 23].contains(&i) {
            if *ch != b'-' {
                return false;
            }
        } else if !(*ch as char).is_ascii_hexdigit() {
            return false;
        }
    }

    true
}

fn guid_to_string_from_raw_bytes(raw: &[u8; 16]) -> String {
    // tld::Guid uses Windows in-memory layout:
    //   data1: u32 little-endian
    //   data2: u16 little-endian
    //   data3: u16 little-endian
    //   data4: [u8; 8]
    //
    // Standard GUID string format:
    //   {xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}

    let data1 = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
    let data2 = u16::from_le_bytes([raw[4], raw[5]]);
    let data3 = u16::from_le_bytes([raw[6], raw[7]]);

    format!(
        "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
        data1,
        data2,
        data3,
        raw[8],
        raw[9],
        raw[10],
        raw[11],
        raw[12],
        raw[13],
        raw[14],
        raw[15]
    )
}

fn provider_name_to_tracelogging_guid_string(provider_name: &str) -> String {
    // TraceLogging dynamic provider names map to stable provider GUIDs.
    // Same provider name => same GUID.
    let guid = tld::Guid::from_name(provider_name);
    let raw = guid.as_bytes_raw();
    guid_to_string_from_raw_bytes(raw)
}

fn parse_field_arg(s: &str) -> Result<DynamicField, String> {
    // Expected format:
    //
    //   Name=type:value
    //
    // Example:
    //
    //   Seq=u32:1
    //   Device=string:FakeBT
    //   Status=bool:true

    let (name, rhs) = s
        .split_once('=')
        .ok_or_else(|| format!("Invalid field '{}': missing '='", s))?;

    let (field_type, value) = rhs
        .split_once(':')
        .ok_or_else(|| format!("Invalid field '{}': missing ':' after type", s))?;

    match field_type {
        "string" => Ok(DynamicField::Str(name.to_string(), value.to_string())),

        "u32" => {
            let parsed = value
                .parse::<u32>()
                .map_err(|e| format!("Invalid u32 in '{}': {}", s, e))?;

            Ok(DynamicField::U32(name.to_string(), parsed))
        }

        "i64" => {
            let parsed = value
                .parse::<i64>()
                .map_err(|e| format!("Invalid i64 in '{}': {}", s, e))?;

            Ok(DynamicField::I64(name.to_string(), parsed))
        }

        "bool" => {
            let parsed = value
                .parse::<bool>()
                .map_err(|e| format!("Invalid bool in '{}': {}", s, e))?;

            Ok(DynamicField::Bool(name.to_string(), parsed))
        }

        other => Err(format!(
            "Unsupported field type '{}' in '{}'. Supported types: string, u32, i64, bool",
            other, s
        )),
    }
}

fn on_event(record: &EventRecord, schema_locator: &SchemaLocator) {
    // This monitor is generic.
    // It does not assume a known event payload schema.
    // It prints provider name and event id only.

    match schema_locator.event_schema(record) {
        Ok(schema) => {
            println!(
                "[recv] provider=\"{}\" event_id={}",
                schema.provider_name(),
                record.event_id()
            );
        }

        Err(err) => {
            println!(
                "[recv] provider=<schema unavailable> event_id={} err={:?}",
                record.event_id(),
                err
            );
        }
    }
}

fn build_provider(name_or_guid: &str) -> Provider {
    // If input looks like a GUID, monitor directly by GUID.
    if looks_like_guid(name_or_guid) {
        println!("[info] monitor by GUID: {}", name_or_guid);

        return Provider::by_guid(name_or_guid)
            .any(u64::MAX)
            .level(5) // Verbose
            .add_callback(on_event)
            .build();
    }

    // First try provider lookup by name.
    // This works for providers discoverable by the system.
    match Provider::by_name(name_or_guid) {
        Ok(builder) => {
            println!("[info] monitor by provider name: {}", name_or_guid);

            builder
                .any(u64::MAX)
                .level(5) // Verbose
                .add_callback(on_event)
                .build()
        }

        Err(err) => {
            // Dynamic TraceLogging providers may not be discoverable by name
            // before the generating process registers them. Fall back to
            // TraceLogging provider-name GUID hash.
            let fallback_guid = provider_name_to_tracelogging_guid_string(name_or_guid);

            println!(
                "[warn] Provider::by_name({}) failed: {:?}",
                name_or_guid, err
            );
            println!(
                "[info] fallback to TraceLogging name-hash GUID: {} -> {}",
                name_or_guid, fallback_guid
            );

            Provider::by_guid(fallback_guid.as_str())
                .any(u64::MAX)
                .level(5) // Verbose
                .add_callback(on_event)
                .build()
        }
    }
}

fn run_monitor(provider_args: &[String]) {
    if provider_args.is_empty() {
        eprintln!("monitor requires at least one provider name or GUID");
        print_usage();
        std::process::exit(1);
    }

    println!("[info] starting realtime ETW monitor");
    println!("[info] providers = {:?}", provider_args);

    let mut trace_builder = UserTrace::new().named("RustEtwMonitor".to_string());

    for provider_arg in provider_args {
        let provider = build_provider(provider_arg);
        trace_builder = trace_builder.enable(provider);
    }

    let _trace = trace_builder
        .start_and_process()
        .expect("start_and_process failed");

    println!("[info] monitoring... press Ctrl+C to stop");

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn write_one_event(provider: &tld::Provider, event_name: &str, fields: &[DynamicField]) {
    let level = tld::Level::Verbose;
    let keyword: u64 = 0x1;

    // If nobody is listening, EventBuilder work is skipped.
    if !provider.enabled(level, keyword) {
        println!("[send] nobody is listening for this level/keyword; event skipped");
        return;
    }

    let mut builder = tld::EventBuilder::new();

    // Start a new dynamic event.
    builder.reset(event_name, level, keyword, 0);

    for field in fields {
        match field {
            DynamicField::Str(name, value) => {
                builder.add_str8(name, value.as_str(), tld::OutType::Default, 0);
            }

            DynamicField::U32(name, value) => {
                builder.add_u32(name, *value, tld::OutType::Default, 0);
            }

            DynamicField::I64(name, value) => {
                builder.add_i64(name, *value, tld::OutType::Default, 0);
            }

            DynamicField::Bool(name, value) => {
                // add_bool32 expects i32, not bool.
                // ETW Bool32 convention: 0 = false, non-zero = true.
                let bool32_value: i32 = if *value { 1 } else { 0 };
                builder.add_bool32(name, bool32_value, tld::OutType::Default, 0);
            }
        }
    }

    // EventBuilder::write expects &tld::Provider.
    let status = builder.write(provider, None, None);

    if status != 0 {
        println!("[send] EventBuilder::write returned Win32 error: {}", status);
    }
}

fn run_generate(args: &[String]) {
    if args.len() < 2 {
        eprintln!(
            "generate requires: <provider_name> <event_name> [--count N] [--interval-ms N] [fields...]"
        );
        print_usage();
        std::process::exit(1);
    }

    let provider_name = &args[0];
    let event_name = &args[1];

    let mut count: u32 = 1;
    let mut interval_ms: u64 = 1000;
    let mut field_args: Vec<String> = Vec::new();

    let mut i = 2;

    while i < args.len() {
        match args[i].as_str() {
            "--count" => {
                if i + 1 >= args.len() {
                    panic!("--count requires a value");
                }

                count = args[i + 1]
                    .parse::<u32>()
                    .unwrap_or_else(|e| panic!("Invalid --count value: {}", e));

                i += 2;
            }

            "--interval-ms" => {
                if i + 1 >= args.len() {
                    panic!("--interval-ms requires a value");
                }

                interval_ms = args[i + 1]
                    .parse::<u64>()
                    .unwrap_or_else(|e| panic!("Invalid --interval-ms value: {}", e));

                i += 2;
            }

            _ => {
                field_args.push(args[i].clone());
                i += 1;
            }
        }
    }

    let mut fields = Vec::new();

    for f in &field_args {
        fields.push(parse_field_arg(f).unwrap_or_else(|e| panic!("{}", e)));
    }

    let provider = Box::pin(tld::Provider::new(
        provider_name,
        &tld::Provider::options(),
    ));

    // register() is unsafe because a registered provider must be properly unregistered.
    // For this short-lived CLI tool, provider cleanup happens when it is dropped.
    unsafe {
        provider.as_ref().register();
    }

    let provider_ref: &tld::Provider = provider.as_ref().get_ref();

    let provider_guid = provider_name_to_tracelogging_guid_string(provider_name);

    println!(
        "[info] provider registered: \"{}\", guid={}, event=\"{}\", count={}, interval_ms={}",
        provider_name, provider_guid, event_name, count, interval_ms
    );

    for n in 0..count {
        println!("[send] event {}/{}", n + 1, count);

        write_one_event(provider_ref, event_name, &fields);

        if n + 1 < count {
            thread::sleep(Duration::from_millis(interval_ms));
        }
    }

    println!("[info] generate done");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "list" => {
            run_list();
        }

        "monitor" => {
            run_monitor(&args[2..]);
        }

        "generate" => {
            run_generate(&args[2..]);
        }

        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}