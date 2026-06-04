use std::env;
use std::process::Command;
use std::thread;
use std::time::Duration;

use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::{TraceTrait, UserTrace};
use ferrisetw::EventRecord;

use tracelogging_dynamic as tld;

// ============================================================================
// winevent
//
// A small ETW command line toolbox for Windows.
//
// Main goals:
//   1. List OS registered ETW providers.
//   2. Find ETW providers by keyword.
//   3. Show active ETW sessions.
//   4. Monitor ETW providers in real time.
//   5. Generate dynamic TraceLogging ETW events for testing.
//   6. Start / stop / delete simple logman trace sessions.
//   7. Decode ETL files with tracerpt.
//
// Cargo.toml:
//
//   [package]
//   name = "winevent"
//   version = "0.1.0"
//   edition = "2021"
//
//   [dependencies]
//   ferrisetw = "1.2"
//   tracelogging_dynamic = "1.2"
//
// Build:
//
//   cargo build
//   cargo build --release
//
// Examples:
//
//   .\target\debug\winevent.exe help
//   .\target\debug\winevent.exe list
//   .\target\debug\winevent.exe find bluetooth
//   .\target\debug\winevent.exe sessions
//   .\target\debug\winevent.exe provider Microsoft-Windows-Kernel-Process
//   .\target\debug\winevent.exe guid Demo.Bluetooth.Test
//   .\target\debug\winevent.exe monitor Demo.Bluetooth.Test
//   .\target\debug\winevent.exe generate Demo.Bluetooth.Test ConnectEvent "Seq=u32:1"
//   .\target\debug\winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl
//   .\target\debug\winevent.exe stop bt
//   .\target\debug\winevent.exe decode bt.etl bt.csv csv
// ============================================================================

// ============================================================================
// DynamicField
//
// This enum represents a runtime-defined ETW field.
//
// The "generate" subcommand accepts fields like:
//
//   Name=string:value
//   Name=u32:123
//   Name=i64:-55
//   Name=bool:true
//
// We parse those strings into this enum, then write them through
// tracelogging_dynamic::EventBuilder.
// ============================================================================
#[derive(Debug, Clone)]
enum DynamicField {
    Str(String, String),
    U32(String, u32),
    I64(String, i64),
    Bool(String, bool),
}

// ============================================================================
// print_usage
//
// Prints the command line help.
//
// Important:
//   Use eprintln!("{}", raw_string)
//   instead of eprintln!(raw_string)
//
// Reason:
//   The help text contains "{GUID}" examples.
//   Rust format strings treat "{}" as placeholders.
//   Passing the raw string as an argument avoids format-string parsing issues.
// ============================================================================
fn print_usage() {
    eprintln!(
        "{}",
        r#"Usage:
  winevent.exe help
  winevent.exe logman-help

  winevent.exe list
  winevent.exe find <keyword>
  winevent.exe sessions
  winevent.exe provider <provider-name-or-guid>
  winevent.exe guid <provider-name>

  winevent.exe monitor <provider-name-or-guid> [provider-name-or-guid] ...

  winevent.exe generate <provider-name> <event-name> [--count N] [--interval-ms N] [fields...]

  winevent.exe record <session-name> <provider-name-or-guid> <etl-file> [flags] [level]
  winevent.exe stop <session-name>
  winevent.exe delete <session-name>
  winevent.exe decode <etl-file> <out-file> [csv]

Examples:
  winevent.exe help
  winevent.exe logman-help

  winevent.exe list
  winevent.exe find bluetooth
  winevent.exe sessions

  winevent.exe provider Microsoft-Windows-Kernel-Process
  winevent.exe provider "{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}"

  winevent.exe guid Demo.Bluetooth.Test

  winevent.exe monitor Demo.Bluetooth.Test
  winevent.exe monitor "{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}"

  winevent.exe generate Demo.Bluetooth.Test ConnectEvent "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"
  winevent.exe generate Demo.Bluetooth.Test ConnectEvent --count 5 --interval-ms 500 "Seq=u32:1"

  winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl
  winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl 0xFFFFFFFF 5

  winevent.exe stop bt
  winevent.exe delete bt

  winevent.exe decode bt.etl bt.xml
  winevent.exe decode bt.etl bt.csv csv

Field format for generate:
  Name=string:value
  Name=u32:value
  Name=i64:value
  Name=bool:true|false
"#
    );
}

// ============================================================================
// run_help
//
// Shows extended help text.
// ============================================================================
fn run_help() {
    print_usage();

    println!(
        "{}",
        r#"
Subcommand summary:

  help
      Show winevent help.

  logman-help
      Run native Windows command:
        logman /?

  list
      Run:
        logman query providers
      This lists OS registered ETW providers and GUIDs.

  find <keyword>
      Run:
        logman query providers
      Then filter provider lines by keyword.
      This is convenient for searching providers like bluetooth, kernel, tcpip.

  sessions
      Run:
        logman query -ets
      This shows currently active Event Trace Sessions.

  provider <provider-name-or-guid>
      Run:
        logman query providers <provider>
      This shows details for one provider when logman can resolve it.

  guid <provider-name>
      Compute TraceLogging name-hash GUID for a dynamic provider name.
      This is useful for dynamic providers such as Demo.Bluetooth.Test.

  monitor <provider-name-or-guid> [...]
      Start a realtime ETW consumer using ferrisetw.
      You can pass one or multiple providers.
      For dynamic TraceLogging providers, if lookup by name fails, winevent
      falls back to the TraceLogging name-hash GUID.

  generate <provider-name> <event-name> [...]
      Generate dynamic TraceLogging events.
      This is useful for testing whether your service can capture ETW events.

  record <session> <provider> <etl-file> [flags] [level]
      Start a simple logman ETW trace session.
      Default flags: 0xFFFFFFFF
      Default level: 5

  stop <session>
      Stop a logman ETW trace session.

  delete <session>
      Delete a logman data collector/session.

  decode <etl-file> <out-file> [csv]
      Decode ETL with tracerpt.
      If the third argument is "csv", output format is CSV.

Recommended workflow:
  1. Find provider:
       winevent.exe find bluetooth

  2. Inspect provider:
       winevent.exe provider <provider-name-or-guid>

  3. Monitor provider:
       winevent.exe monitor <provider-name-or-guid>

  4. Generate a test event:
       winevent.exe generate Demo.Bluetooth.Test ConnectEvent "Seq=u32:1"

  5. Record to ETL:
       winevent.exe record bt <provider-name-or-guid> bt.etl

  6. Stop recording:
       winevent.exe stop bt

  7. Decode ETL:
       winevent.exe decode bt.etl bt.csv csv
"#
    );
}

// ============================================================================
// run_external_command
//
// Helper for calling external Windows tools such as:
//
//   logman
//   tracerpt
//
// It prints stdout and stderr directly.
// This keeps the wrapper simple and transparent.
// ============================================================================
fn run_external_command(program: &str, args: &[&str]) {
    let output = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run '{} {:?}': {}", program, args, e));

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        eprintln!(
            "[error] command failed: {} {:?}, exit code: {:?}",
            program,
            args,
            output.status.code()
        );
    }
}

// ============================================================================
// Basic logman wrapper subcommands
// ============================================================================

fn run_logman_help() {
    // Equivalent to:
    //   logman /?
    run_external_command("logman", &["/?"]);
}

fn run_list() {
    // Equivalent to:
    //   logman query providers
    //
    // Shows all OS registered ETW providers and GUIDs.
    run_external_command("logman", &["query", "providers"]);
}

fn run_sessions() {
    // Equivalent to:
    //   logman query -ets
    //
    // Shows currently active Event Trace Sessions.
    run_external_command("logman", &["query", "-ets"]);
}

fn run_provider(args: &[String]) {
    // Query details for a specific provider.
    //
    // Equivalent to:
    //   logman query providers <provider>
    //
    // Example:
    //   winevent.exe provider Microsoft-Windows-Kernel-Process
    //   winevent.exe provider "{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}"

    if args.is_empty() {
        eprintln!("provider requires a provider name or GUID");
        print_usage();
        std::process::exit(1);
    }

    run_external_command("logman", &["query", "providers", args[0].as_str()]);
}

fn run_find(args: &[String]) {
    // Search providers by keyword.
    //
    // This runs:
    //   logman query providers
    //
    // Then filters lines in Rust.
    //
    // Example:
    //   winevent.exe find bluetooth

    if args.is_empty() {
        eprintln!("find requires a keyword");
        print_usage();
        std::process::exit(1);
    }

    let keyword = args[0].to_lowercase();

    let output = Command::new("logman")
        .args(["query", "providers"])
        .output()
        .unwrap_or_else(|e| panic!("failed to run 'logman query providers': {}", e));

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.to_lowercase().contains(&keyword) {
            println!("{}", line);
        }
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

// ============================================================================
// GUID helpers
//
// TraceLogging dynamic providers use a stable GUID generated from provider name.
// This is important because dynamic providers may not be discoverable by name
// through Provider::by_name before the generator process registers them.
// ============================================================================

fn looks_like_guid(s: &str) -> bool {
    // Accept both:
    //   22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716
    //   {22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}

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
    //
    //   data1: u32 little-endian
    //   data2: u16 little-endian
    //   data3: u16 little-endian
    //   data4: [u8; 8]
    //
    // Standard GUID string format:
    //
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
    // Generate the stable TraceLogging GUID from a provider name.
    //
    // Example:
    //   Demo.Bluetooth.Test -> {some-stable-guid}
    //
    // This is useful when monitoring dynamic providers.

    let guid = tld::Guid::from_name(provider_name);
    let raw = guid.as_bytes_raw();
    guid_to_string_from_raw_bytes(raw)
}

fn run_guid(args: &[String]) {
    // Print TraceLogging name-hash GUID for one or more provider names.
    //
    // Example:
    //   winevent.exe guid Demo.Bluetooth.Test

    if args.is_empty() {
        eprintln!("guid requires at least one provider name");
        print_usage();
        std::process::exit(1);
    }

    for provider_name in args {
        let guid = provider_name_to_tracelogging_guid_string(provider_name);
        println!("{} {}", provider_name, guid);
    }
}

// ============================================================================
// Field parser for "generate"
// ============================================================================

fn parse_field_arg(s: &str) -> Result<DynamicField, String> {
    // Expected format:
    //
    //   Name=type:value
    //
    // Examples:
    //
    //   Seq=u32:1
    //   Device=string:FakeBT
    //   Rssi=i64:-55
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

// ============================================================================
// Realtime ETW monitor
//
// ferrisetw creates a realtime ETW trace session and invokes a callback
// for each event.
// ============================================================================

fn on_event(record: &EventRecord, schema_locator: &SchemaLocator) {
    // This callback is intentionally generic.
    //
    // It does not assume any known event payload schema.
    // That is important because "monitor" can target any provider.
    //
    // For now, print:
    //   provider name
    //   event id
    //
    // Later you can extend this to parse specific provider fields.

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
            .level(5)
            .add_callback(on_event)
            .build();
    }

    // First try provider lookup by name.
    //
    // This works for many OS-registered providers, for example:
    //   Microsoft-Windows-Kernel-Process
    //
    // But it may fail for dynamic TraceLogging providers such as:
    //   Demo.Bluetooth.Test
    match Provider::by_name(name_or_guid) {
        Ok(builder) => {
            println!("[info] monitor by provider name: {}", name_or_guid);

            builder
                .any(u64::MAX)
                .level(5)
                .add_callback(on_event)
                .build()
        }

        Err(err) => {
            // Dynamic TraceLogging providers may not be discoverable by name.
            // In that case, fallback to the TraceLogging provider-name GUID.
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
                .level(5)
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

    let mut trace_builder = UserTrace::new().named("WineventRealtimeMonitor".to_string());

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

// ============================================================================
// Dynamic ETW event generator
//
// Uses tracelogging_dynamic to generate TraceLogging events with runtime
// provider name, event name, and fields.
// ============================================================================

fn write_one_event(provider: &tld::Provider, event_name: &str, fields: &[DynamicField]) {
    let level = tld::Level::Verbose;
    let keyword: u64 = 0x1;

    // If no ETW session is listening for this provider/level/keyword,
    // provider.enabled(...) returns false.
    //
    // In that case, we skip building the event to reduce overhead.
    if !provider.enabled(level, keyword) {
        println!("[send] nobody is listening for this level/keyword; event skipped");
        return;
    }

    let mut builder = tld::EventBuilder::new();

    // Start a new dynamic TraceLogging event.
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
                // ETW Bool32 convention:
                //   0 = false
                //   non-zero = true
                let bool32_value: i32 = if *value { 1 } else { 0 };
                builder.add_bool32(name, bool32_value, tld::OutType::Default, 0);
            }
        }
    }

    // Send event to ETW.
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

    // Register the provider with ETW.
    //
    // Safety:
    //   register() is unsafe because a registered provider must be properly
    //   unregistered before unload.
    //
    // For this short-lived CLI process, the provider is dropped when the process exits.
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

// ============================================================================
// logman record / stop / delete / tracerpt decode helpers
// ============================================================================

fn run_record(args: &[String]) {
    // Start a simple ETW trace session using logman.
    //
    // Usage:
    //   winevent.exe record <session-name> <provider> <etl-file> [flags] [level]
    //
    // Example:
    //   winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl
    //   winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl 0xFFFFFFFF 5
    //
    // Defaults:
    //   flags = 0xFFFFFFFF
    //   level = 5

    if args.len() < 3 {
        eprintln!("record requires: <session-name> <provider> <etl-file> [flags] [level]");
        print_usage();
        std::process::exit(1);
    }

    let session = &args[0];
    let provider = &args[1];
    let etl_file = &args[2];

    let flags = if args.len() >= 4 {
        args[3].as_str()
    } else {
        "0xFFFFFFFF"
    };

    let level = if args.len() >= 5 {
        args[4].as_str()
    } else {
        "5"
    };

    run_external_command(
        "logman",
        &[
            "create",
            "trace",
            session.as_str(),
            "-p",
            provider.as_str(),
            flags,
            level,
            "-o",
            etl_file.as_str(),
            "-ets",
        ],
    );
}

fn run_stop(args: &[String]) {
    // Stop an active ETW session.
    //
    // Equivalent to:
    //   logman stop <session> -ets

    if args.is_empty() {
        eprintln!("stop requires a session name");
        print_usage();
        std::process::exit(1);
    }

    run_external_command("logman", &["stop", args[0].as_str(), "-ets"]);
}

fn run_delete(args: &[String]) {
    // Delete a logman data collector/session.
    //
    // Equivalent to:
    //   logman delete <session>

    if args.is_empty() {
        eprintln!("delete requires a session name");
        print_usage();
        std::process::exit(1);
    }

    run_external_command("logman", &["delete", args[0].as_str()]);
}

fn run_decode(args: &[String]) {
    // Decode an ETL file with tracerpt.
    //
    // Usage:
    //   winevent.exe decode <etl-file> <out-file> [csv]
    //
    // Examples:
    //   winevent.exe decode bt.etl bt.xml
    //   winevent.exe decode bt.etl bt.csv csv

    if args.len() < 2 {
        eprintln!("decode requires: <etl-file> <out-file> [csv]");
        print_usage();
        std::process::exit(1);
    }

    let etl_file = &args[0];
    let out_file = &args[1];

    if args.len() >= 3 && args[2].eq_ignore_ascii_case("csv") {
        run_external_command(
            "tracerpt",
            &[etl_file.as_str(), "-o", out_file.as_str(), "-of", "CSV"],
        );
    } else {
        run_external_command("tracerpt", &[etl_file.as_str(), "-o", out_file.as_str()]);
    }
}

// ============================================================================
// main
//
// Dispatch command line subcommands.
// ============================================================================

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "help" | "-h" | "--help" | "/?" => {
            run_help();
        }

        "logman-help" => {
            run_logman_help();
        }

        "list" => {
            run_list();
        }

        "find" => {
            run_find(&args[2..]);
        }

        "sessions" => {
            run_sessions();
        }

        "provider" => {
            run_provider(&args[2..]);
        }

        "guid" => {
            run_guid(&args[2..]);
        }

        "monitor" => {
            run_monitor(&args[2..]);
        }

        "generate" => {
            run_generate(&args[2..]);
        }

        "record" => {
            run_record(&args[2..]);
        }

        "stop" => {
            run_stop(&args[2..]);
        }

        "delete" => {
            run_delete(&args[2..]);
        }

        "decode" => {
            run_decode(&args[2..]);
        }

        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}