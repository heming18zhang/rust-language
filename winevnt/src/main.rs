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
// A small ETW (Event Tracing for Windows) toolbox for Windows.
//
// This program provides these practical subcommands:
//
//   help
//   logman-help
//   list
//   find <keyword>
//   sessions
//   provider <provider-name-or-guid>
//   guid <provider-name>
//   monitor <provider-name-or-guid> [...]
//   generate <provider-name> <event-name> [...]
//   record <session-name> <provider> <etl-file> [flags] [level]
//   stop <session-name>
//   delete <session-name>
//   decode <etl-file> <out-file> [csv]
//
// Main goals:
//   1. List OS-registered ETW providers with logman.
//   2. Search providers by keyword.
//   3. Show active ETW sessions.
//   4. Monitor ETW providers in real time with ferrisetw.
//   5. Generate dynamic TraceLogging ETW events for testing.
//   6. Record ETW sessions to ETL using logman.
//   7. Stop/delete sessions and decode ETL files.
//
// Important fixes in this file:
//
//   A) PowerShell bare "{GUID}" issue
//      In PowerShell, a bare "{...}" token is not always passed as a normal
//      process argument. Sometimes PowerShell rewrites the execution into an
//      -encodedCommand form, which means the program does NOT receive the GUID
//      directly. This file includes recovery logic for that scenario.
//
//   B) Invalid GUID string issue
//      Provider::by_guid(...) is strict about GUID string format.
//      This file normalizes GUID strings before passing them to by_guid.
// ============================================================================

/// Represents one runtime-defined field for the "generate" subcommand.
///
/// Supported command-line field formats:
///
///   Name=string:value
///   Name=u32:value
///   Name=i64:value
///   Name=bool:true
///   Name=bool:false
///
/// Examples:
///
///   Seq=u32:1
///   Device=string:FakeBT
///   Rssi=i64:-55
///   Connected=bool:true
#[derive(Debug, Clone)]
enum DynamicField {
    Str(String, String),
    U32(String, u32),
    I64(String, i64),
    Bool(String, bool),
}

// ============================================================================
// Help / usage text
// ============================================================================

/// Print one-screen usage information.
///
/// Important formatting note:
///
/// We intentionally use:
///
///     eprintln!("{}", raw_text)
///
/// instead of:
///
///     eprintln!(raw_text)
///
/// because the help text contains "{GUID}" examples. If the raw string is used
/// directly as a format string, Rust would try to interpret braces as format
/// placeholders.
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
  winevent.exe monitor "DB25B328-A6F6-444F-9D97-A50E20217D16"
  winevent.exe monitor "{DB25B328-A6F6-444F-9D97-A50E20217D16}"

  winevent.exe generate Demo.Bluetooth.Test ConnectEvent "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"
  winevent.exe generate Demo.Bluetooth.Test ConnectEvent --count 5 --interval-ms 500 "Seq=u32:1"

  winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl
  winevent.exe stop bt
  winevent.exe delete bt
  winevent.exe decode bt.etl bt.csv csv

Field format for generate:
  Name=string:value
  Name=u32:value
  Name=i64:value
  Name=bool:true|false
"#
    );
}

/// Print extended help.
///
/// This exists separately from print_usage() so the short usage and the longer
/// explanation remain organized and easy to maintain.
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

  sessions
      Run:
        logman query -ets
      Shows currently active Event Trace Sessions.

  provider <provider-name-or-guid>
      Run:
        logman query providers <provider>
      Shows details for one provider.

  guid <provider-name>
      Compute the TraceLogging name-hash GUID for a dynamic provider name.

  monitor <provider-name-or-guid> [...]
      Start a realtime ETW consumer using ferrisetw.

  generate <provider-name> <event-name> [...]
      Generate dynamic TraceLogging events for testing.

  record <session> <provider> <etl-file> [flags] [level]
      Start a simple logman ETW trace session.

  stop <session>
      Stop a logman ETW trace session.

  delete <session>
      Delete a logman data collector/session.

  decode <etl-file> <out-file> [csv]
      Decode ETL with tracerpt.

PowerShell note:
  In PowerShell, this is WRONG:

      winevent.exe monitor {DB25B328-A6F6-444F-9D97-A50E20217D16}

  Use one of these instead:

      winevent.exe monitor "DB25B328-A6F6-444F-9D97-A50E20217D16"
      winevent.exe monitor "{DB25B328-A6F6-444F-9D97-A50E20217D16}"

  This program still tries to recover if PowerShell mangles the argument,
  but quoting the GUID is still the correct way to call it.
"#
    );
}

// ============================================================================
// External command helpers
// ============================================================================

/// Run an external program and print stdout/stderr as-is.
///
/// This helper is used for wrappers around native Windows tools such as:
///
///   logman
///   tracerpt
///
/// The goal is to keep these wrappers simple and transparent:
/// let the native tool do the work, and forward output directly to the user.
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
// Basic logman/tracerpt wrappers
// ============================================================================

/// Show native logman help.
fn run_logman_help() {
    run_external_command("logman", &["/?"]);
}

/// List OS registered ETW providers and GUIDs.
///
/// Equivalent to:
///
///   logman query providers
fn run_list() {
    run_external_command("logman", &["query", "providers"]);
}

/// Show active Event Trace Sessions.
///
/// Equivalent to:
///
///   logman query -ets
fn run_sessions() {
    run_external_command("logman", &["query", "-ets"]);
}

/// Query details for one provider.
///
/// Equivalent to:
///
///   logman query providers <provider>
fn run_provider(args: &[String]) {
    if args.is_empty() {
        eprintln!("provider requires a provider name or GUID");
        print_usage();
        std::process::exit(1);
    }

    run_external_command("logman", &["query", "providers", args[0].as_str()]);
}

/// Search OS registered providers by keyword.
///
/// This runs:
///
///   logman query providers
///
/// and filters the output lines in Rust.
///
/// Example:
///
///   winevent.exe find bluetooth
fn run_find(args: &[String]) {
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
// ============================================================================

/// Remove leading/trailing curly braces from a GUID string.
///
/// Example:
///
///   "{xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}" -> "xxxxxxxx-...."
fn trim_guid_braces(s: &str) -> &str {
    s.trim_matches(|c| c == '{' || c == '}')
}

/// Check whether a string looks like a standard 36-char GUID
/// with hyphens:
///
///   xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
fn looks_like_guid(s: &str) -> bool {
    let trimmed = trim_guid_braces(s);

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

/// Check whether a string looks like a 32-char GUID without hyphens.
///
/// Example:
///
///   DB25B328A6F6444F9D97A50E20217D16
fn looks_like_guid_loose(s: &str) -> bool {
    let trimmed = trim_guid_braces(s);

    if trimmed.len() != 32 {
        return false;
    }

    trimmed.chars().all(|c| c.is_ascii_hexdigit())
}

/// Normalize a GUID string into the standard 36-char hyphenated form
/// without curly braces.
///
/// Supported inputs:
///
///   DB25B328-A6F6-444F-9D97-A50E20217D16
///   {DB25B328-A6F6-444F-9D97-A50E20217D16}
///   DB25B328A6F6444F9D97A50E20217D16
///
/// Returns:
///
///   db25b328-a6f6-444f-9d97-a50e20217d16
fn normalize_guid_string(s: &str) -> Option<String> {
    let trimmed = trim_guid_braces(s).trim();

    if looks_like_guid(trimmed) {
        return Some(trimmed.to_string());
    }

    if looks_like_guid_loose(trimmed) {
        let g = format!(
            "{}-{}-{}-{}-{}",
            &trimmed[0..8],
            &trimmed[8..12],
            &trimmed[12..16],
            &trimmed[16..20],
            &trimmed[20..32]
        );
        return Some(g);
    }

    None
}

/// Convert raw Windows in-memory GUID bytes into a standard GUID string.
///
/// Output form is:
///
///   xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
///
/// Note:
/// We intentionally do NOT return curly braces, because Provider::by_guid(...)
/// is happier with the plain hyphenated form.
fn guid_to_string_from_raw_bytes(raw: &[u8; 16]) -> String {
    let data1 = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
    let data2 = u16::from_le_bytes([raw[4], raw[5]]);
    let data3 = u16::from_le_bytes([raw[6], raw[7]]);

    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
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

/// Compute the stable TraceLogging provider GUID from a provider name.
///
/// This is useful for dynamic TraceLogging providers such as:
///
///   Demo.Bluetooth.Test
///
/// because those providers may not always be discoverable by name before the
/// generator process registers them.
fn provider_name_to_tracelogging_guid_string(provider_name: &str) -> String {
    let guid = tld::Guid::from_name(provider_name);
    let raw = guid.as_bytes_raw();
    guid_to_string_from_raw_bytes(raw)
}

/// Print one or more TraceLogging name-hash GUIDs for provider names.
///
/// Example:
///
///   winevent.exe guid Demo.Bluetooth.Test
fn run_guid(args: &[String]) {
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
// PowerShell -encodedCommand recovery helpers
// ============================================================================

/// Try to find a GUID-looking substring inside arbitrary text.
///
/// This is used after decoding a PowerShell -encodedCommand payload.
/// We scan the text for either:
///
///   36-char GUID with hyphens
///   38-char GUID with braces
///   32-char GUID without hyphens
fn try_extract_guid_from_text(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();

    for start in 0..chars.len() {
        for len in [38usize, 36, 32] {
            if start + len <= chars.len() {
                let candidate: String = chars[start..start + len].iter().collect();
                if let Some(g) = normalize_guid_string(&candidate) {
                    return Some(g);
                }
            }
        }
    }

    None
}

/// Decode a base64 string with a very small local decoder.
///
/// We implement our own decoder here to avoid adding extra crate dependencies.
/// This is enough for the PowerShell -encodedCommand recovery path.
fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }

    let cleaned: Vec<u8> = input
        .bytes()
        .filter(|b| !b" \r\n\t".contains(b))
        .collect();

    if cleaned.is_empty() {
        return Err("empty base64".to_string());
    }

    let mut out = Vec::new();
    let mut i = 0usize;

    while i < cleaned.len() {
        let c0 = *cleaned.get(i).ok_or("invalid base64 length")?;
        let c1 = *cleaned.get(i + 1).ok_or("invalid base64 length")?;
        let c2 = *cleaned.get(i + 2).unwrap_or(&b'=');
        let c3 = *cleaned.get(i + 3).unwrap_or(&b'=');

        let v0 = val(c0).ok_or("invalid base64 char")? as u32;
        let v1 = val(c1).ok_or("invalid base64 char")? as u32;
        let v2 = if c2 == b'=' {
            0
        } else {
            val(c2).ok_or("invalid base64 char")? as u32
        };
        let v3 = if c3 == b'=' {
            0
        } else {
            val(c3).ok_or("invalid base64 char")? as u32
        };

        let n = (v0 << 18) | (v1 << 12) | (v2 << 6) | v3;

        out.push(((n >> 16) & 0xFF) as u8);
        if c2 != b'=' {
            out.push(((n >> 8) & 0xFF) as u8);
        }
        if c3 != b'=' {
            out.push((n & 0xFF) as u8);
        }

        i += 4;
    }

    Ok(out)
}

/// If the argument list looks like a PowerShell -encodedCommand rewrite,
/// try to decode it and recover the GUID.
///
/// Observed broken argument pattern:
///
///   -encodedCommand <base64>
///   -inputFormat xml
///   -outputFormat text
///
/// We try to:
///   1. locate -encodedCommand
///   2. base64 decode it
///   3. interpret bytes as UTF-16LE PowerShell text
///   4. extract a GUID from the decoded text
fn try_decode_powershell_encoded_command_to_guid(args: &[String]) -> Option<String> {
    if args.is_empty() {
        return None;
    }

    let mut i = 0usize;
    while i < args.len() {
        let arg = args[i].to_ascii_lowercase();
        if arg == "-encodedcommand" && i + 1 < args.len() {
            let b64 = &args[i + 1];

            if let Ok(bytes) = decode_base64(b64) {
                // PowerShell -encodedCommand is usually UTF-16LE.
                if bytes.len() % 2 == 0 {
                    let utf16: Vec<u16> = bytes
                        .chunks_exact(2)
                        .map(|b| u16::from_le_bytes([b[0], b[1]]))
                        .collect();

                    if let Ok(text) = String::from_utf16(&utf16) {
                        if let Some(guid) = try_extract_guid_from_text(&text) {
                            return Some(guid);
                        }
                    }
                }
            }
        }

        i += 1;
    }

    None
}

/// Recover monitor arguments if PowerShell mangled them.
///
/// Normal case:
///
///   winevent.exe monitor "GUID"
///
/// Broken PowerShell case:
///
///   winevent.exe monitor {GUID}
///
/// may arrive as:
///
///   -encodedCommand ...
///
/// In that case we try to recover the GUID from the encoded command.
/// If we can recover it, we replace the whole provider list with that GUID.
fn recover_monitor_args(args: &[String]) -> Vec<String> {
    if args.is_empty() {
        return vec![];
    }

    if args.iter().any(|a| a.eq_ignore_ascii_case("-encodedCommand")) {
        if let Some(guid) = try_decode_powershell_encoded_command_to_guid(args) {
            return vec![guid];
        }
    }

    args.to_vec()
}

// ============================================================================
// Field parser for "generate"
// ============================================================================

/// Parse one dynamic field argument.
///
/// Input format:
///
///   Name=type:value
///
/// Supported types:
///
///   string
///   u32
///   i64
///   bool
fn parse_field_arg(s: &str) -> Result<DynamicField, String> {
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
// ============================================================================

/// Generic ETW event callback.
///
/// This callback intentionally does NOT assume a known provider schema.
/// Since this tool may monitor any provider, the safest default is to print:
///
///   provider name
///   event id
///
/// You can later extend this for specific provider payload parsing if needed.
fn on_event(record: &EventRecord, schema_locator: &SchemaLocator) {
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

/// Build one ferrisetw Provider subscription from a provider name or GUID.
///
/// Logic:
///
/// 1. If the input can be normalized into a GUID string, use by_guid(...)
/// 2. Otherwise try by_name(...)
/// 3. If by_name(...) fails, assume it may be a dynamic TraceLogging provider
///    name and fallback to the TraceLogging name-hash GUID.
///
/// This is especially useful for dynamic providers like:
///
///   Demo.Bluetooth.Test
fn build_provider(name_or_guid: &str) -> Provider {
    // First, try to interpret the input as a GUID.
    if let Some(guid) = normalize_guid_string(name_or_guid) {
        println!("[info] monitor by GUID: {}", guid);

        return Provider::by_guid(guid.as_str())
            .any(u64::MAX)
            .level(5)
            .add_callback(on_event)
            .build();
    }

    // Otherwise, try provider lookup by name.
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
            // If lookup by name fails, fallback to TraceLogging name-hash GUID.
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

/// Start realtime monitoring.
///
/// This function also applies the PowerShell-argument recovery logic before
/// building providers.
fn run_monitor(provider_args: &[String]) {
    let provider_args = recover_monitor_args(provider_args);

    if provider_args.is_empty() {
        eprintln!("monitor requires at least one provider name or GUID");
        print_usage();
        std::process::exit(1);
    }

    println!("[info] starting realtime ETW monitor");
    println!("[info] providers = {:?}", provider_args);

    let mut trace_builder = UserTrace::new().named("WineventRealtimeMonitor".to_string());

    for provider_arg in &provider_args {
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
// Dynamic TraceLogging generator
// ============================================================================

/// Build and send one dynamic TraceLogging ETW event.
///
/// This uses tracelogging_dynamic, so both provider name and event fields
/// can be defined at runtime.
fn write_one_event(provider: &tld::Provider, event_name: &str, fields: &[DynamicField]) {
    let level = tld::Level::Verbose;
    let keyword: u64 = 0x1;

    // If no ETW session is listening for this provider/level/keyword,
    // skip event construction to avoid unnecessary overhead.
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
                // ETW Bool32 convention:
                //   0 = false
                //   non-zero = true
                let bool32_value: i32 = if *value { 1 } else { 0 };
                builder.add_bool32(name, bool32_value, tld::OutType::Default, 0);
            }
        }
    }

    let status = builder.write(provider, None, None);

    if status != 0 {
        println!("[send] EventBuilder::write returned Win32 error: {}", status);
    }
}

/// Generate one or more dynamic events.
///
/// Example:
///
///   winevent.exe generate Demo.Bluetooth.Test ConnectEvent "Seq=u32:1"
///   winevent.exe generate Demo.Bluetooth.Test ConnectEvent --count 5 --interval-ms 500 "Seq=u32:1"
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
    // For this short-lived CLI process, provider cleanup happens when the
    // process exits and the provider is dropped.
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

/// Start a simple logman ETW trace session.
///
/// Usage:
///
///   winevent.exe record <session-name> <provider> <etl-file> [flags] [level]
///
/// Defaults:
///
///   flags = 0xFFFFFFFF
///   level = 5
fn run_record(args: &[String]) {
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

    let level = if args.len() >= 5 { args[4].as_str() } else { "5" };

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

/// Stop an active logman ETW session.
///
/// Equivalent to:
///
///   logman stop <session> -ets
fn run_stop(args: &[String]) {
    if args.is_empty() {
        eprintln!("stop requires a session name");
        print_usage();
        std::process::exit(1);
    }

    run_external_command("logman", &["stop", args[0].as_str(), "-ets"]);
}

/// Delete a logman data collector/session.
///
/// Equivalent to:
///
///   logman delete <session>
fn run_delete(args: &[String]) {
    if args.is_empty() {
        eprintln!("delete requires a session name");
        print_usage();
        std::process::exit(1);
    }

    run_external_command("logman", &["delete", args[0].as_str()]);
}

/// Decode an ETL file using tracerpt.
///
/// Usage:
///
///   winevent.exe decode <etl-file> <out-file> [csv]
///
/// If the third argument is "csv", output format is CSV.
/// Otherwise tracerpt default output is used.
fn run_decode(args: &[String]) {
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