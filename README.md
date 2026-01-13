# KeyboardRemapperR

Lightweight, device-specific keyboard remapper for Windows, written in Rust.

This is a minimal, single-file implementation of a keyboard remapper that allows you to set different key mappings for each connected keyboard.

## Features

- **Device-Specific Mappings**: Apply different key remapping rules to each keyboard.
- **Multiple Mapping Modes**: Supports `remap`, `swap`, and `disable`.
- **JSON Configuration**: Save and load your settings from a `config.json` file.
- **CLI Interface**: Manage your devices and mappings through a simple command-line interface.
- **Minimalist Design**: Implemented in a single Rust file for simplicity and performance.

## Quick Start

1.  **List Devices**: Identify your keyboards.

    ```bash
    keyboard-remapper-r list
    ```

2.  **Set Mapping**: Swap `CapsLock` and `LCtrl` on your external keyboard (e.g., VID:PID `04FE:0021`).

    ```bash
    keyboard-remapper-r set 04FE:0021 CapsLock LCtrl --mode swap
    keyboard-remapper-r set 04FE:0021 LCtrl CapsLock --mode swap
    ```

3.  **Save Configuration**: Save your settings to `config.json`.

    ```bash
    keyboard-remapper-r save
    ```

4.  **Start Service**: Run the remapper in the background.

    ```bash
    keyboard-remapper-r start
    ```

## Commands

- `list`: List all connected keyboards and their current mappings.
- `set <DEVICE_ID> <FROM> <TO> [--mode <MODE>]`: Set a key mapping for a device.
- `remove <DEVICE_ID> <FROM>`: Remove a key mapping.
- `show <DEVICE_ID>`: Show the mappings for a specific device.
- `save [--output <PATH>]`: Save the current configuration to a file.
- `load [--input <PATH>]`: Load a configuration from a file.
- `start`: Start the remapping service (requires admin rights on Windows).
- `stop`: Stop the remapping service.

## Installation

### For Users

**Download Pre-built Binary** (Recommended):

1. Go to the [Releases](https://github.com/tkykszk/KeyboardRemapperR/releases) page
2. Download `keyboard-remapper-r-windows.zip`
3. Extract the ZIP file
4. Run `keyboard-remapper-r.exe`

### For Developers

**Prerequisites**:
- [Rust](https://www.rust-lang.org/) (1.70 or later)
- [Git](https://git-scm.com/)
- Windows 10/11 (64-bit)

**Development Setup**:

1. **Clone the repository**:

   ```bash
   git clone https://github.com/tkykszk/KeyboardRemapperR.git
   cd KeyboardRemapperR
   ```

2. **Install dependencies**:

   ```bash
   cargo fetch
   ```

3. **Build (Debug)**:

   ```bash
   cargo build
   ```

   The debug executable will be at `target/debug/keyboard-remapper-r.exe`

4. **Build (Release)**:

   ```bash
   cargo build --release
   ```

   The release executable will be at `target/release/keyboard-remapper-r.exe`

5. **Run tests**:

   ```bash
   cargo test
   ```

6. **Run the application**:

   ```bash
   cargo run -- list
   ```

**Development Workflow**:

```bash
# Format code
cargo fmt

# Check code
cargo check

# Run clippy (linter)
cargo clippy

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_add_device

# Build and run in one command
cargo run -- list
```

**Project Structure**:

```
KeyboardRemapperR/
├── src/
│   └── main.rs          # Single-file implementation (295 lines)
├── tests/
│   ├── e2e_test.ps1            # Full E2E test suite
│   └── e2e_test_simple.ps1     # Simple E2E test
├── .github/workflows/
│   └── build-and-test.yml      # CI/CD configuration
├── Cargo.toml           # Rust dependencies
└── README.md            # This file
```

**Dependencies**:

- `clap`: Command-line argument parser
- `serde`: Serialization framework
- `serde_json`: JSON support
- `winapi`: Windows API bindings

**Contributing**:

Pull requests are welcome! Please ensure:
- Code is formatted with `cargo fmt`
- All tests pass with `cargo test`
- No clippy warnings with `cargo clippy`

## License

This project is licensed under the MIT License.
