# Silent Mode and Background Tasks

## Overview

The `--silent` (or `-s`) flag suppresses all console output, making MonitorConfig ideal for background automation, Task Scheduler jobs, and startup scripts.

## Usage

Add `--silent` or `-s` as a global flag to any command:

```bash
monitorconfig set-brightness 50 --primary --silent
monitorconfig set-vcp 0x14 11 -d "Lenovo L22i-40" -s
monitorconfig get-brightness --primary --silent
```

## When to Use Silent Mode

- **Task Scheduler**: Prevent unnecessary logging in scheduled tasks
- **Startup Scripts**: Avoid console windows during system boot
- **Batch Files**: Clean execution without output clutter
- **Background Services**: Silent operation for monitoring/automation tools

## Console Window Flash Prevention

### Problem

Even with `--silent`, Windows may briefly show a console window when launching the executable from Task Scheduler or other GUI contexts.

### Solution 1: Windows GUI Subsystem Build

Build MonitorConfig without console window support using Cargo features:

```bash
# Windows build
cargo build --release --features gui-subsystem

# Cross-compile from Linux
cargo build --release --target x86_64-pc-windows-gnu --features gui-subsystem
```

The `gui-subsystem` feature flag tells Cargo to compile without console window support.

**Note**: With this build, you won't see error messages in console. Only use for production automation where errors are logged elsewhere.

### Solution 2: VBScript Wrapper

Create `run_silent.vbs`:

```vbscript
Set WshShell = CreateObject("Wscript.Shell")
WshShell.Run "monitorconfig.exe set-brightness 50 --primary --silent", 0, True
```

Then use the VBScript in Task Scheduler instead of the EXE directly.

### Solution 3: PowerShell Hidden Window

```powershell
$proc = Start-Process -FilePath "monitorconfig.exe" `
    -ArgumentList "set-vcp 0x14 11 -d 'Monitor Name' --silent" `
    -WindowStyle Hidden `
    -Wait `
    -PassThru

exit $proc.ExitCode
```

## Task Scheduler Setup Example

### Basic Configuration

1. **Program/script**: `C:\Path\To\monitorconfig.exe`
2. **Add arguments**: `set-brightness 50 --primary --silent`
3. **Start in**: `C:\Path\To\`

### Advanced Settings

- ✓ Run whether user is logged on or not
- ✓ Run with highest privileges (if needed for some monitors)
- ✓ Hidden (if using GUI subsystem build)

### Trigger Examples

**On Startup**:

- Trigger: At startup
- Delay: 10 seconds (give monitors time to initialize)

**Time-based**:

- Trigger: Daily at 9:00 AM
- Action: Set brightness to 100%

**On Unlock**:

- Trigger: On workstation unlock
- Action: Restore preferred settings

## Exit Codes

MonitorConfig uses standard exit codes:

- `0`: Success
- `1`: Error (check stderr if not in GUI subsystem mode)

In silent mode, errors are still written to stderr unless you're using the GUI subsystem build.

## Error Handling in Scripts

Even in silent mode, you can check for errors:

### Batch File

```batch
@echo off
monitorconfig.exe set-brightness 50 --primary --silent
if %ERRORLEVEL% NEQ 0 (
    echo Error setting brightness >> C:\logs\monitor.log
)
```

### PowerShell

```powershell
& monitorconfig.exe set-brightness 50 --primary --silent
if ($LASTEXITCODE -ne 0) {
    Write-EventLog -LogName Application -Source "MonitorConfig" `
        -EventId 1001 -EntryType Error `
        -Message "Failed to set brightness"
}
```

## Best Practices

1. **Always use `--silent`** with Task Scheduler to avoid unnecessary output
2. **Test commands manually first** without `--silent` to verify they work
3. **Use absolute paths** in Task Scheduler for reliability
4. **Add delays** for startup tasks (monitors may not be ready immediately)
5. **Log errors** to a file when running in fully silent mode (GUI subsystem)
6. **Consider VBScript wrapper** for complete invisibility without recompiling

## Comparison of Methods

| Method              | Window Flash     | See Errors   | Requires Rebuild |
| ------------------- | ---------------- | ------------ | ---------------- |
| `--silent` flag     | Yes (brief)      | Yes (stderr) | No               |
| VBScript wrapper    | No               | No           | No               |
| PowerShell Hidden   | Yes (very brief) | Yes          | No               |
| GUI subsystem build | No               | No           | Yes              |

## Examples

### Morning Brightness Automation

```batch
REM morning_bright.bat
monitorconfig.exe set-brightness 100 --primary --silent
monitorconfig.exe set-vcp 0x14 6500 --primary --silent
```

### Evening Dim Automation

```batch
REM evening_dim.bat
monitorconfig.exe set-brightness 30 --primary --silent
monitorconfig.exe set-vcp 0x14 3000 --primary --silent
```

### Multi-Monitor Setup

```batch
REM setup_monitors.bat
monitorconfig.exe set-brightness 80 -d "\\.\DISPLAY1" --silent
monitorconfig.exe set-brightness 80 -d "\\.\DISPLAY2" --silent
monitorconfig.exe set-contrast 75 -d "\\.\DISPLAY1" --silent
monitorconfig.exe set-contrast 75 -d "\\.\DISPLAY2" --silent
```

Schedule these scripts with Task Scheduler for automatic monitor management throughout the day.
