# MonitorConfig - Copilot Instructions

## Project Overview

A native Windows CLI tool for controlling external monitors via DDC/CI protocol. Written in Rust, this is a complete rewrite of a PowerShell module focused on performance and cross-compilation capabilities.

**Target Platform**: Windows 10/11 (uses dxva2.dll and user32.dll)  
**Rust Edition**: 2024 (requires Rust 1.93.0+)

## Architecture

### Module Structure

- **[main.rs](../src/main.rs)**: Entry point - minimal error handling wrapper
- **[cli.rs](../src/cli.rs)**: Clap-based command definitions and routing logic (437 lines)
- **[monitor.rs](../src/monitor.rs)**: `Monitor` trait, `PhysicalMonitor` struct, and monitor enumeration/discovery functions
- **[native.rs](../src/native.rs)**: FFI bindings to Windows APIs (`dxva2.dll` for DDC/CI, `user32.dll` for enumeration)
- **[vcp.rs](../src/vcp.rs)**: VCP (VESA Command Protocol) feature codes and scanning logic (1110 lines of VCP code definitions)
- **[error.rs](../src/error.rs)**: `thiserror`-based error types (`MonitorError` enum)

### Key Design Patterns

**Monitor Discovery Flow**:

1. `MonitorEnumerator::enumerate()` → uses Win32 `EnumDisplayMonitors` callback
2. For each HMONITOR → `get_physical_monitors()` → gets `PHYSICAL_MONITOR` structs via `GetPhysicalMonitorsFromHMONITOR`
3. `PhysicalMonitor::new()` wraps the handle and extracts friendly name from UTF-16 description
4. Helper functions: `enumerate_monitors()`, `find_monitor()`, `get_primary_monitor()`

**FFI Safety**: All unsafe Windows API calls are isolated in [native.rs](../src/native.rs). Use `#[link(name = "dxva2")]` extern blocks for DDC/CI functions not in `windows-sys`.

**Error Handling**: Return `Result<T>` (aliased to `Result<T, MonitorError>`) from all fallible operations. Use `thiserror` for error definitions. Windows API failures (return 0) should convert to appropriate `MonitorError` variants.

**Monitor Selection**: Commands accept `--primary` flag OR `--device <name>`. Device name can be display device path (e.g., `\\.\DISPLAY1`) or friendly name (e.g., "Dell U2723DE"). The `get_monitor()` helper in [cli.rs](../src/cli.rs#L238) handles this logic.

## Build and Cross-Compilation

**Windows Build**: `cargo build --release` or `build.bat`  
**Linux → Windows Cross-Compilation**: `build.sh` uses MinGW-w64 with target `x86_64-pc-windows-gnu`

**Release Profile Optimizations** in [Cargo.toml](../Cargo.toml#L15-L19):

```toml
[profile.release]
opt-level = 3
strip = true
lto = true
codegen-units = 1
```

## Critical Conventions

1. **VCP Code Format**: Always use hex format (e.g., `0x10` for brightness). The CLI uses a custom `parse_hex` parser for VCP code arguments.

2. **JSON Output**: All read commands support `--json` flag. Use `serde_json::to_string_pretty()` for consistent formatting.

3. **Brightness/Contrast Values**: Range is 0-100, enforced by monitor hardware (not the tool).

4. **VCP Scanning**: The `scan-vcp` command tests all codes 0x00-0xFF and returns only supported ones (similar to PowerShell's `Get-MonitorVCPResponse -All`). Silently ignore unsupported codes.

5. **Device Name Workaround**: [monitor.rs:45](../src/monitor.rs#L45) uses `format!("DISPLAY_{:p}", hmonitor)` as a placeholder because `windows-sys` MONITORINFOEXW struct layout requires additional parsing. TODO: Extract actual device name from display device.

6. **Handle Cleanup**: `PhysicalMonitor` implements `Drop` to call `DestroyPhysicalMonitor`. Always ensure RAII cleanup for monitor handles.

## Testing and Validation

**Manual Testing Requirements**:

- Requires physical external monitor with DDC/CI enabled (check monitor OSD settings)
- Test on Windows 10/11 - DDC/CI is Windows-specific
- Use `monitorconfig list` to verify detection before testing other commands

**Common Test Commands**:

```bash
monitorconfig list --json
monitorconfig get-brightness --primary
monitorconfig scan-vcp --primary --json
```

## Dependencies and External APIs

- **windows-sys 0.61.2**: Minimal Windows API bindings for `Win32_Foundation`, `Win32_Graphics_Gdi`
- **dxva2.dll**: DDC/CI functions (manually linked, not in windows-sys)
- **clap 4.5**: CLI parsing with derive macros
- **serde/serde_json**: JSON serialization for all data structures

**Why Not windows-rs**: Chose `windows-sys` for smaller binary size (critical for CLI tool distribution).

## Common Pitfalls

1. **Testing Without Monitor**: DDC/CI commands will fail on laptop built-in displays or monitors with DDC/CI disabled. Always test with external monitors.

2. **Cross-Compilation Setup**: Linux developers need MinGW-w64 toolchain (`gcc-mingw-w64-x86-64` on Debian/Ubuntu) plus `rustup target add x86_64-pc-windows-gnu`.

3. **VCP Code Support Varies**: Not all monitors support all VCP codes. Use `scan-vcp` to discover supported codes for a specific monitor model.

4. **Unsafe Code Justification**: All `unsafe` blocks are for FFI - ensure proper error checking (Windows APIs return 0 on failure) and handle cleanup in Drop implementations.
