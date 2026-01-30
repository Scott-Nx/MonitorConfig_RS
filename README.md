# MonitorConfig - Rust CLI

A native Windows CLI tool for managing monitor settings via DDC/CI and WMI, written in Rust. This is a complete rewrite of the original PowerShell module.

## Features

- **List Monitors**: Enumerate all connected monitors
- **Brightness Control**: Get and set monitor brightness levels
- **Contrast Control**: Get and set monitor contrast levels
- **VCP Support**: Full VCP (VESA Command Protocol) feature access
- **Monitor Capabilities**: Query monitor capabilities string
- **Settings Management**: Save current settings or reset to factory defaults
- **JSON Output**: All commands support JSON output for scripting

## Requirements

- Windows 10/11 (uses dxva2.dll and user32.dll)
- Rust 1.93.0+ (Edition 2024)
- External monitors must support DDC/CI (enable in monitor OSD if needed)

## Building

### Prerequisites

Install Rust from [rustup.rs](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build Release Version

```bash
cd rust-cli
cargo build --release
```

The compiled binary will be at `target/release/monitorconfig.exe`

### Install Globally

```bash
cargo install --path .
```

## Usage

### List All Monitors

```bash
monitorconfig list

# JSON output
monitorconfig list --json
```

### Get Brightness

```bash
# Primary monitor
monitorconfig get-brightness --primary

# Specific monitor
monitorconfig get-brightness --device "\\.\DISPLAY1"

# JSON output
monitorconfig get-brightness --primary --json
```

### Set Brightness

```bash
# Set primary monitor to 50%
monitorconfig set-brightness 50 --primary

# Set specific monitor
monitorconfig set-brightness 75 --device "\\.\DISPLAY1"
```

### Get Contrast

```bash
monitorconfig get-contrast --primary
```

### Set Contrast

```bash
monitorconfig set-contrast 60 --primary
```

### Get VCP Feature

```bash
# Get brightness (VCP code 0x10)
monitorconfig get-vcp 0x10 --primary

# Get power mode (VCP code 0xD6)
monitorconfig get-vcp 0xD6 --primary --json
```

### Set VCP Feature

```bash
# Set brightness via VCP
monitorconfig set-vcp 0x10 75 --primary

# Turn off monitor (power mode = 4)
monitorconfig set-vcp 0xD6 4 --primary
```

### List Known VCP Codes

```bash
# Show reference list of common VCP codes
monitorconfig list-vcp

# JSON output
monitorconfig list-vcp --json
```

### Scan Monitor for All Supported VCP Codes

```bash
# Scan primary monitor for all supported VCP codes
monitorconfig scan-vcp --primary

# Scan specific monitor
monitorconfig scan-vcp --device "\\.\DISPLAY1"

# JSON output for scripting
monitorconfig scan-vcp --primary --json
```

**Note**: `list-vcp` shows a reference list of common VCP codes, while `scan-vcp` actively queries your monitor to discover which codes it actually supports (similar to PowerShell's `Get-MonitorVCPResponse -All`).

### Get Monitor Capabilities

```bash
monitorconfig get-capabilities --primary
```

### Save Settings

```bash
# Save current settings to monitor's memory
monitorconfig save-settings --primary
```

### Reset to Factory Defaults

```bash
# Reset all settings
monitorconfig reset-defaults --primary

# Reset only color settings
monitorconfig reset-defaults --primary --color-only
```

## Background Tasks / Task Scheduler

When running MonitorConfig from Windows Task Scheduler or other background automation tools, you may want to suppress console output and prevent the command window from flashing.

### Silent Mode

Use the global `--silent` (or `-s`) flag to suppress all console output:

```bash
# Silent mode - no output
monitorconfig set-vcp -d "Lenovo L22i-40" 0x14 11 --silent

# Also works with short form
monitorconfig set-brightness 50 --primary -s
```

This flag works with all commands and is especially useful for:

- Task Scheduler automated tasks
- Startup scripts
- Batch files
- Background automation

### Complete Console Window Suppression

For Windows builds, you can compile the binary to completely hide the console window using Cargo features. This prevents even the brief flash when running from Task Scheduler.

**Build with GUI subsystem** (no console window):

```bash
cargo build --release --features gui-subsystem
```

For cross-compilation from Linux:

```bash
cargo build --release --target x86_64-pc-windows-gnu --features gui-subsystem
```

**Note**: GUI subsystem builds won't show any console output or errors. Only use this for production automation where errors are logged elsewhere. For development and testing, use the standard build without the feature flag.

## Common VCP Codes

| Code | Name                 | Description                   |
| ---- | -------------------- | ----------------------------- |
| 0x10 | Brightness           | Luminance of the image        |
| 0x12 | Contrast             | Contrast of the image         |
| 0x14 | Color Temperature    | Select color temperature      |
| 0x16 | Red Video Gain       | Red video gain (drive)        |
| 0x18 | Green Video Gain     | Green video gain (drive)      |
| 0x1A | Blue Video Gain      | Blue video gain (drive)       |
| 0x60 | Input Source         | Select input source           |
| 0x62 | Audio Speaker Volume | Audio speaker volume          |
| 0x8D | Audio Mute           | Audio mute/unmute             |
| 0xD6 | Power Mode           | DPM/DPMS status (1=On, 4=Off) |

## Examples

### Set Multiple Monitors to Same Brightness

```bash
# List monitors and get device names
monitorconfig list

# Set each monitor
monitorconfig set-brightness 80 --device "\\.\DISPLAY1"
monitorconfig set-brightness 80 --device "\\.\DISPLAY2"
```

### Turn Off All Monitors

```bash
# Using VCP power mode command (value 4 = Off)
monitorconfig set-vcp 0xD6 4 --device "\\.\DISPLAY1"
monitorconfig set-vcp 0xD6 4 --device "\\.\DISPLAY2"
```

### Query Monitor Information

```bash
# Get all info in JSON format
monitorconfig list --json > monitors.json
monitorconfig get-brightness --primary --json >> monitors.json
monitorconfig get-capabilities --primary >> capabilities.txt
```

## Technical Details

### Dependencies (Latest Versions)

- `windows = "0.62.2"` - Windows API bindings
- `clap = "4.5.55"` - Command-line argument parsing
- `anyhow = "1.0.100"` - Error handling
- `serde = "1.0.228"` - Serialization framework
- `serde_json = "1.0.149"` - JSON support
- `thiserror = "2.0.18"` - Error derive macros

### Architecture

The tool is structured into several modules:

- **native**: Low-level Windows API bindings (dxva2.dll, user32.dll)
- **monitor**: Monitor abstraction and enumeration
- **vcp**: VCP (Video Control Panel) feature implementation
- **cli**: Command-line interface using clap
- **error**: Centralized error handling

### DDC/CI Support

DDC/CI (Display Data Channel Command Interface) allows software control of monitor settings. Most modern external monitors support it, but it may need to be enabled in the monitor's OSD (On-Screen Display) menu.

#### Troubleshooting DDC/CI

1. Check if DDC/CI is enabled in monitor OSD
2. Try different cable types (DisplayPort usually works better than HDMI)
3. Update monitor firmware if available
4. Some USB-C docks may not support DDC/CI

## Performance

The Rust implementation provides several advantages over the PowerShell module:

- **Faster startup**: No .NET CLR initialization overhead
- **Lower memory usage**: Compiled binary vs interpreted PowerShell
- **Native performance**: Direct Windows API calls without managed wrappers
- **Smaller distribution**: Single ~1MB executable (release build)

## Migration from PowerShell Module

### Command Mapping

| PowerShell Cmdlet        | Rust CLI Command               |
| ------------------------ | ------------------------------ |
| `Get-Monitor`            | `monitorconfig list`           |
| `Get-MonitorBrightness`  | `monitorconfig get-brightness` |
| `Set-MonitorBrightness`  | `monitorconfig set-brightness` |
| `Get-MonitorVCPResponse` | `monitorconfig get-vcp`        |
| `Set-MonitorVCPValue`    | `monitorconfig set-vcp`        |
| `Get-MonitorDetails`     | `monitorconfig list --json`    |
| `Save-MonitorSettings`   | `monitorconfig save-settings`  |
| `Reset-MonitorSettings`  | `monitorconfig reset-defaults` |

### Parameter Mapping

| PowerShell Parameter | Rust CLI Option   |
| -------------------- | ----------------- |
| `-DeviceName`        | `--device`        |
| `-Primary`           | `--primary`       |
| `-VCPCode`           | First positional  |
| `-Value`             | Second positional |

## License

This project follows the same license as the original MonitorConfig PowerShell module (see LICENSE file).

## Contributing

Contributions are welcome! Areas for improvement:

- [ ] Add WMI support for laptop internal displays
- [ ] Add configuration file support
- [ ] Add monitor profile save/restore
- [ ] Cross-platform support exploration (Linux/macOS DDC support)

## Acknowledgments

Based on the [MonitorConfig PowerShell Module](https://github.com/MartinGC94/MonitorConfig) by Martin Gräßlin.
