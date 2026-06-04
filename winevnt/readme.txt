winevent README
===============

Overview
--------
winevent is a small Windows ETW (Event Tracing for Windows) command-line toolbox written in Rust.

It is intended for development and testing scenarios where you want to:

1. List OS-registered ETW providers.
2. Search ETW providers by keyword.
3. View active ETW sessions.
4. Monitor one or more ETW providers in real time.
5. Generate test ETW events using a dynamic TraceLogging provider.
6. Record ETW events to an ETL file using logman.
7. Stop or delete logman sessions.
8. Decode ETL files using tracerpt.

This tool is especially useful when testing whether a service can capture ETW events.


Project Layout
--------------
Typical project layout:

    winevent/
      Cargo.toml
      src/
        main.rs
      README.txt


Cargo.toml
----------
Use this Cargo.toml:

    [package]
    name = "winevent"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    ferrisetw = "1.2"
    tracelogging_dynamic = "1.2"

After changing the package name to winevent, the generated executable will be:

    target\debug\winevent.exe

or, for release build:

    target\release\winevent.exe


Build
-----
Debug build:

    cargo build

Release build:

    cargo build --release

Clean and rebuild:

    cargo clean
    cargo build


Subcommands
-----------
The tool supports these subcommands:

    help
    logman-help
    list
    find <keyword>
    sessions
    provider <provider-name-or-guid>
    guid <provider-name>
    monitor <provider-name-or-guid> [provider-name-or-guid] ...
    generate <provider-name> <event-name> [--count N] [--interval-ms N] [fields...]
    record <session-name> <provider-name-or-guid> <etl-file> [flags] [level]
    stop <session-name>
    delete <session-name>
    decode <etl-file> <out-file> [csv]


1. help
-------
Show winevent help.

Example:

    winevent.exe help

Also supported:

    winevent.exe -h
    winevent.exe --help
    winevent.exe /?


2. logman-help
--------------
Runs the native Windows command:

    logman /?

Example:

    winevent.exe logman-help

Use this if you want to see the built-in logman help directly from winevent.


3. list
-------
Lists OS-registered ETW providers and GUIDs.

Internally, this runs:

    logman query providers

Example:

    winevent.exe list

You can pipe it to findstr:

    winevent.exe list | findstr /i bluetooth


4. find <keyword>
-----------------
Searches ETW providers by keyword.

Internally, this runs:

    logman query providers

Then it filters the output in Rust.

Examples:

    winevent.exe find bluetooth
    winevent.exe find kernel
    winevent.exe find tcpip
    winevent.exe find power

This is often the fastest way to find a provider name or GUID.


5. sessions
-----------
Lists active Event Trace Sessions.

Internally, this runs:

    logman query -ets

Example:

    winevent.exe sessions

Use this to check whether a logman ETW session is currently running.


6. provider <provider-name-or-guid>
-----------------------------------
Shows information about one ETW provider.

Internally, this runs:

    logman query providers <provider>

Examples:

    winevent.exe provider Microsoft-Windows-Kernel-Process
    winevent.exe provider "{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}"

This is useful after you find a provider using list or find.


7. guid <provider-name>
-----------------------
Computes the TraceLogging name-hash GUID for a dynamic provider name.

Example:

    winevent.exe guid Demo.Bluetooth.Test

Why this matters:

Dynamic TraceLogging providers may not always be discoverable by provider name before the generator process registers them. In that case, monitor can use the TraceLogging name-hash GUID instead.

Example workflow:

    winevent.exe guid Demo.Bluetooth.Test
    winevent.exe monitor "{GUID_FROM_OUTPUT}"


8. monitor <provider-name-or-guid> [...]
----------------------------------------
Starts a realtime ETW monitor using ferrisetw.

Examples:

    winevent.exe monitor Microsoft-Windows-Kernel-Process
    winevent.exe monitor "{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}"
    winevent.exe monitor Demo.Bluetooth.Test

You can monitor multiple providers:

    winevent.exe monitor Demo.Bluetooth.Test Microsoft-Windows-Kernel-Process

Behavior:

- If the argument looks like a GUID, winevent monitors by GUID.
- If it looks like a provider name, winevent first tries Provider::by_name.
- If Provider::by_name fails, winevent falls back to the TraceLogging name-hash GUID.

The monitor currently prints generic event metadata:

    [recv] provider="..." event_id=...

It does not decode arbitrary payload fields because different providers have different schemas.


9. generate <provider-name> <event-name> [fields...]
----------------------------------------------------
Generates dynamic TraceLogging ETW events.

Basic example:

    winevent.exe generate Demo.Bluetooth.Test ConnectEvent "Seq=u32:1"

More fields:

    winevent.exe generate Demo.Bluetooth.Test ConnectEvent "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"

Generate multiple events:

    winevent.exe generate Demo.Bluetooth.Test ConnectEvent --count 5 --interval-ms 500 "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"

Supported field formats:

    Name=string:value
    Name=u32:value
    Name=i64:value
    Name=bool:true
    Name=bool:false

Examples:

    Seq=u32:1
    Device=string:FakeBT
    Rssi=i64:-55
    Connected=bool:true

Important behavior:

If no ETW session is listening for the provider, level, and keyword, the generator may print:

    [send] nobody is listening for this level/keyword; event skipped

This is expected. Start monitor first, then run generate in another terminal.

Recommended test flow:

Terminal 1:

    winevent.exe monitor Demo.Bluetooth.Test

Terminal 2:

    winevent.exe generate Demo.Bluetooth.Test ConnectEvent --count 5 --interval-ms 500 "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"


10. record <session> <provider> <etl-file> [flags] [level]
-----------------------------------------------------------
Starts a simple ETW recording session using logman.

Internally, this runs a command similar to:

    logman create trace <session> -p <provider> <flags> <level> -o <etl-file> -ets

Default values:

    flags = 0xFFFFFFFF
    level = 5

Examples:

    winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl
    winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl 0xFFFFFFFF 5
    winevent.exe record proc "{22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716}" proc.etl

After recording, stop the session with:

    winevent.exe stop bt


11. stop <session>
------------------
Stops an active logman ETW session.

Internally, this runs:

    logman stop <session> -ets

Example:

    winevent.exe stop bt


12. delete <session>
--------------------
Deletes a logman data collector/session.

Internally, this runs:

    logman delete <session>

Example:

    winevent.exe delete bt

Use this if you created a session and want to clean it up.


13. decode <etl-file> <out-file> [csv]
--------------------------------------
Decodes an ETL file using tracerpt.

XML/default output:

    winevent.exe decode bt.etl bt.xml

CSV output:

    winevent.exe decode bt.etl bt.csv csv

Internally, CSV mode runs:

    tracerpt bt.etl -o bt.csv -of CSV


Common Workflows
----------------

Workflow A: Find and monitor a Bluetooth provider

    winevent.exe find bluetooth
    winevent.exe provider Microsoft-Windows-Bluetooth-BTHPORT
    winevent.exe monitor Microsoft-Windows-Bluetooth-BTHPORT


Workflow B: Generate and monitor a test dynamic provider

Terminal 1:

    winevent.exe monitor Demo.Bluetooth.Test

Terminal 2:

    winevent.exe generate Demo.Bluetooth.Test ConnectEvent --count 5 --interval-ms 500 "Seq=u32:1" "Status=bool:true" "Device=string:FakeBT"


Workflow C: Record ETW to an ETL file

    winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl

Trigger the scenario you want to test, then stop:

    winevent.exe stop bt

Decode:

    winevent.exe decode bt.etl bt.csv csv


Workflow D: Check active sessions

    winevent.exe sessions


Troubleshooting
---------------

Problem: monitor Demo.Bluetooth.Test says provider by name was not found.

Explanation:

Demo.Bluetooth.Test is a dynamic TraceLogging provider. It may not be registered as an OS provider before the generating process starts.

Solution:

winevent automatically falls back to the TraceLogging name-hash GUID. You can also view the GUID manually:

    winevent.exe guid Demo.Bluetooth.Test


Problem: generate says nobody is listening.

Message:

    [send] nobody is listening for this level/keyword; event skipped

Explanation:

The dynamic provider checks whether any ETW session is enabled for the provider, level, and keyword. If nothing is listening, it skips the event.

Solution:

Start monitor first:

    winevent.exe monitor Demo.Bluetooth.Test

Then run generate in another terminal.


Problem: logman command fails.

Possible reasons:

1. You are not running from a Windows environment.
2. logman is not available in PATH.
3. The operation needs administrator privileges.
4. The session already exists.
5. The provider name or GUID is invalid.

Useful checks:

    winevent.exe logman-help
    winevent.exe sessions
    winevent.exe list


Problem: record says the session already exists.

Solution:

Stop and delete the old session:

    winevent.exe stop <session-name>
    winevent.exe delete <session-name>

Then create it again.


Notes
-----

- winevent monitor is realtime and prints to console.
- winevent record writes ETW events to an ETL file.
- winevent generate creates dynamic TraceLogging events for testing.
- winevent list/find/provider/sessions are wrappers around logman.
- winevent decode is a wrapper around tracerpt.
- monitor output is generic by design because arbitrary ETW providers have different payload schemas.


Recommended Commands Quick Reference
------------------------------------

    winevent.exe help
    winevent.exe list
    winevent.exe find bluetooth
    winevent.exe provider <provider-name-or-guid>
    winevent.exe sessions
    winevent.exe guid Demo.Bluetooth.Test
    winevent.exe monitor Demo.Bluetooth.Test
    winevent.exe generate Demo.Bluetooth.Test ConnectEvent "Seq=u32:1"
    winevent.exe record bt Microsoft-Windows-Bluetooth-BTHPORT bt.etl
    winevent.exe stop bt
    winevent.exe delete bt
    winevent.exe decode bt.etl bt.csv csv

End of README
