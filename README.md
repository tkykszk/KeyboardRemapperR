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

## Building from Source

1.  **Install Rust**: If you don't have Rust, install it from [rust-lang.org](https://www.rust-lang.org/).
2.  **Build**: Clone the repository and build the release binary.

    ```bash
    git clone https://github.com/your-username/KeyboardRemapperR.git
    cd KeyboardRemapperR
    cargo build --release
    ```

3.  **Run**: The executable will be at `target/release/keyboard-remapper-r`.

## License

This project is licensed under the MIT License.
