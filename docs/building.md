# Building MonitorConfig

## Quick Start

### Standard Build (with console window)

```bash
# Windows
build.bat

# Linux (cross-compile)
./build.sh
```

Or directly with Cargo:

```bash
# Windows native
cargo build --release

# Linux cross-compile
cargo build --release --target x86_64-pc-windows-gnu
```

### GUI Subsystem Build (no console window)

For Task Scheduler and background automation where you don't want any console window flash:

```bash
# Windows
build.bat --gui

# Linux (cross-compile)
./build.sh --gui
```

Or directly with Cargo:

```bash
# Windows native
cargo build --release --features gui-subsystem

# Linux cross-compile
cargo build --release --target x86_64-pc-windows-gnu --features gui-subsystem
```

## Build Variants

| Build Type    | Command                                          | Console Window | Use Case                                          |
| ------------- | ------------------------------------------------ | -------------- | ------------------------------------------------- |
| Standard      | `cargo build --release`                          | Yes (normal)   | Interactive use, testing, debugging               |
| GUI Subsystem | `cargo build --release --features gui-subsystem` | No             | Task Scheduler, background tasks, startup scripts |

## Feature Flags

MonitorConfig uses Cargo features for build-time configuration:

### `gui-subsystem`

Compiles the application with `windows_subsystem = "windows"`, which tells Windows to treat it as a GUI application rather than a console application.

**Pros:**

- No console window appears at all (not even a flash)
- Perfect for Task Scheduler and background automation
- Cleaner user experience for scheduled tasks

**Cons:**

- No console output visible (stdout/stderr are discarded)
- Error messages won't be visible
- Only use with `--silent` flag or when errors are logged elsewhere

**When to use:**

- Running from Task Scheduler
- Startup scripts
- Background automation where visibility isn't needed
- Any scenario where console window flash is undesirable

**When NOT to use:**

- Interactive/manual usage
- Debugging
- Development
- When you need to see error messages

## Cross-Compilation from Linux

### Prerequisites

Install MinGW-w64 toolchain:

```bash
# Debian/Ubuntu
sudo apt-get install gcc-mingw-w64-x86-64

# Arch Linux
sudo pacman -S mingw-w64-gcc

# Fedora
sudo dnf install mingw64-gcc
```

Add Rust Windows target:

```bash
rustup target add x86_64-pc-windows-gnu
```

### Build Commands

```bash
# Standard build
cargo build --release --target x86_64-pc-windows-gnu

# GUI subsystem build
cargo build --release --target x86_64-pc-windows-gnu --features gui-subsystem
```

Binary location: `target/x86_64-pc-windows-gnu/release/monitorconfig.exe`

## Development Builds

For development with faster compile times:

```bash
# Debug build (faster compilation, larger binary, includes debug symbols)
cargo build

# Run tests
cargo test

# Check without building
cargo check
```

## Build Profiles

Defined in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
strip = true         # Strip symbols for smaller binary
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization at cost of compile time
```

This produces a ~1MB optimized binary.

## Troubleshooting

### MinGW Linker Errors

If you see linker errors during cross-compilation:

1. Ensure MinGW-w64 is installed
2. Add to `~/.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

### Feature Flag Not Working

Ensure you're using `--features` not `-features`:

```bash
# Correct
cargo build --release --features gui-subsystem

# Wrong
cargo build --release -features gui-subsystem
```

### Binary Size

Release builds are optimized for size. If you need to reduce further:

```toml
[profile.release]
opt-level = "z"  # Optimize for size
```

But note that `opt-level = 3` usually produces faster code.

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build standard
        run: cargo build --release
      - name: Build GUI subsystem
        run: cargo build --release --features gui-subsystem
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: monitorconfig-builds
          path: |
            target/release/monitorconfig.exe
```

### Build Both Variants Script

Create a script to build both versions:

```bash
#!/bin/bash
# build-all.sh

set -e

echo "Building standard version..."
cargo build --release --target x86_64-pc-windows-gnu

echo "Copying standard build..."
cp target/x86_64-pc-windows-gnu/release/monitorconfig.exe \
   target/monitorconfig.exe

echo "Building GUI subsystem version..."
cargo build --release --target x86_64-pc-windows-gnu --features gui-subsystem

echo "Copying GUI build..."
cp target/x86_64-pc-windows-gnu/release/monitorconfig.exe \
   target/monitorconfig-gui.exe

echo "Done! Binaries:"
echo "  Standard:      target/monitorconfig.exe"
echo "  GUI Subsystem: target/monitorconfig-gui.exe"
```

## Installation

After building, install globally:

```bash
# Standard version
cargo install --path .

# GUI subsystem version
cargo install --path . --features gui-subsystem
```

Or copy the binary manually:

```bash
# Copy to a directory in your PATH
cp target/release/monitorconfig.exe C:/Tools/
```
