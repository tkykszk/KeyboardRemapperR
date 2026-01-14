# KeyboardRemapperR

[![Phase 3 Tests](https://github.com/tkykszk/KeyboardRemapperR/actions/workflows/test.yml/badge.svg)](https://github.com/tkykszk/KeyboardRemapperR/actions/workflows/test.yml)

Lightweight, device-specific keyboard remapper for Windows, written in Rust.

KeyboardRemapperR allows you to set different key mappings for each connected keyboard using Windows Raw Input API and Low-level keyboard hooks.

## ✨ Features

- **Device-Specific Mappings**: Apply different key remapping rules to each keyboard by VID/PID
- **Multiple Mapping Modes**: Supports `remap`, `swap`, and `disable`
- **Real-time Key Remapping**: Uses Raw Input API and Low-level keyboard hooks for instant response
- **JSON Configuration**: Save and load your settings from a `config.json` file
- **CLI Interface**: Manage your devices and mappings through a simple command-line interface
- **Minimalist Design**: Single-file implementation for simplicity and performance
- **No GUI Required**: Lightweight CLI-only tool (GUI version planned for Phase 6)

## 🚀 Quick Start

### 1. Download

Download the latest release from the [Releases](https://github.com/tkykszk/KeyboardRemapperR/releases) page.

### 2. List Devices

Identify your keyboards by VID/PID:

```bash
keyboard-remapper-r.exe list
```

Output example:
```
Connected Keyboards:
  - 04FE:0021 \\?\HID#VID_04FE&PID_0021#... [Configured]
    Mappings: 2
  - 046D:C52B \\?\HID#VID_046D&PID_C52B#...
    Mappings: 0
```

### 3. Set Mappings

Remap CapsLock to LCtrl on your external keyboard:

```bash
keyboard-remapper-r.exe set 04FE:0021 CapsLock LCtrl --mode remap
```

Swap two keys:

```bash
keyboard-remapper-r.exe set 04FE:0021 CapsLock LCtrl --mode swap
```

Disable a key:

```bash
keyboard-remapper-r.exe set 04FE:0021 CapsLock None --mode disable
```

### 4. Save Configuration

Save your settings to `config.json`:

```bash
keyboard-remapper-r.exe save
```

### 5. Start Service

Run the remapper (requires administrator privileges):

```bash
keyboard-remapper-r.exe start
```

**Note**: You must run as administrator for the Low-level keyboard hook to work properly.

## 📋 Commands

| Command | Description |
|---------|-------------|
| `list` | List all connected keyboards and their current mappings |
| `set <DEVICE_ID> <FROM> <TO> [--mode <MODE>]` | Set a key mapping for a device |
| `remove <DEVICE_ID> <FROM>` | Remove a key mapping |
| `show <DEVICE_ID>` | Show the mappings for a specific device |
| `save [--output <PATH>]` | Save the current configuration to a file |
| `load [--input <PATH>]` | Load a configuration from a file |
| `start` | Start the remapping service (requires admin rights) |
| `stop` | Stop the remapping service |

### Mapping Modes

- **remap**: Map one key to another (one-way)
- **swap**: Swap two keys (bidirectional, automatically creates reverse mapping)
- **disable**: Disable a key (use `None` as target)

## 🎯 Supported Keys

KeyboardRemapperR supports 90+ keys including:

**Alphabetic**: A-Z  
**Numeric**: 0-9, Numpad 0-9  
**Function Keys**: F1-F24  
**Modifiers**: LCtrl, RCtrl, LAlt, RAlt, LShift, RShift, LWin, RWin  
**Special Keys**: CapsLock, Tab, Enter, Space, Backspace, Escape, etc.  
**Navigation**: Arrow keys, Home, End, PageUp, PageDown, Insert, Delete  
**Symbols**: Semicolon, Plus, Comma, Minus, Period, Slash, Tilde, etc.

See the full list in the source code (`vk_to_key_name` function).

## 🏗️ Architecture

KeyboardRemapperR uses a hybrid approach combining Raw Input API and Low-level keyboard hooks:

1. **Raw Input API**: Identifies which keyboard device generated the input (by VID/PID)
2. **Low-level Keyboard Hook**: Suppresses the original key press
3. **SendInput API**: Sends the remapped key press
4. **Infinite Loop Prevention**: Uses a marker (`dwExtraInfo`) to prevent re-processing sent keys

### Flow Diagram

```
User presses key
    ↓
Raw Input receives WM_INPUT message
    ↓
Extract device handle → VID/PID
    ↓
Convert VK code → Key name
    ↓
Search mapping (Device ID + Key name)
    ↓
Add VK code to suppression list
    ↓
Low-level hook suppresses original key
    ↓
SendInput sends remapped key (with marker)
    ↓
Low-level hook detects marker → Pass through
    ↓
Result: Remapped key is pressed
```

## 📦 Installation

### For Users

**Download Pre-built Binary** (Recommended):

1. Go to the [Releases](https://github.com/tkykszk/KeyboardRemapperR/releases) page
2. Download `keyboard-remapper-r-windows.zip` from the latest release
3. Extract the ZIP file
4. Run `keyboard-remapper-r.exe` as administrator

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

2. **Build (Release)**:

   ```bash
   cargo build --release
   ```

   The release executable will be at `target/release/keyboard-remapper-r.exe`

3. **Run tests**:

   ```bash
   cargo test
   ```

4. **Run the application**:

   ```bash
   cargo run --release -- list
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
cargo run --release -- list
```

## 📁 Project Structure

```
KeyboardRemapperR/
├── src/
│   └── main.rs                      # Single-file implementation (~1500 lines)
├── tests/
│   ├── e2e_tests.rs                 # E2E test suite
│   ├── phase3_unit_tests.rs         # Phase 3 unit tests
│   └── phase3_integration_tests.rs  # Phase 3 integration tests
├── scripts/
│   ├── run_tests.ps1                # Test automation script (Windows)
│   └── run_tests.sh                 # Test automation script (Linux)
├── .github/workflows/
│   └── test.yml                     # CI/CD configuration
├── Cargo.toml                       # Rust dependencies
├── README.md                        # This file
├── PHASE1_COMPLETION_REPORT.md      # Phase 1 report
├── PHASE2_COMPLETION_REPORT.md      # Phase 2 report
├── PHASE3_COMPLETION_REPORT.md      # Phase 3 report
└── IMPLEMENTATION_TASKS.md          # Implementation roadmap
```

## 🧪 Testing

KeyboardRemapperR includes comprehensive test suites:

### Run Tests Locally

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_vk_to_key_name
```

### Automated Testing

Tests are automatically run on GitHub Actions for every push and pull request.

**Test Coverage**:
- Unit tests: 30+ tests
- Integration tests: 10+ tests
- Performance tests: 5+ tests
- E2E tests: 6+ tests

See `.github/workflows/test.yml` for CI/CD configuration.

## 🔧 Dependencies

- **clap**: Command-line argument parser
- **serde**: Serialization framework
- **serde_json**: JSON support
- **winapi**: Windows API bindings (Raw Input, Low-level hooks, SendInput)

## 🗺️ Roadmap

### Phase 1-3: Core Functionality ✅ (Completed)

- ✅ Device detection and VID/PID extraction
- ✅ Raw Input API integration
- ✅ VK code conversion table (90+ keys)
- ✅ Low-level keyboard hook
- ✅ SendInput API integration
- ✅ Remap/Swap/Disable modes

### Phase 4: Service and Management (Planned)

- ⏳ Background service (Windows Service)
- ⏳ Hot reload configuration
- ⏳ Logging functionality
- ⏳ Start/Stop commands

### Phase 5: Testing and Documentation (Planned)

- ⏳ Complete E2E test suite
- ⏳ User guide
- ⏳ Developer documentation
- ⏳ Performance benchmarks

### Phase 6: Advanced Features (Planned)

- ⏳ Modifier key combinations (Ctrl+A, etc.)
- ⏳ Macro functionality
- ⏳ Profile switching
- ⏳ GUI version (WinForms/WPF)
- ⏳ Performance optimization

See `IMPLEMENTATION_TASKS.md` for detailed task breakdown.

## 🐛 Known Issues

### v0.1.0-alpha1 (Current)

- Background service is not implemented yet (use `start` command in foreground)
- Hot reload is not implemented (restart required for config changes)
- GUI is not available (CLI only)
- Modifier key combinations are not supported yet

See [Issues](https://github.com/tkykszk/KeyboardRemapperR/issues) for the full list.

## 🤝 Contributing

Pull requests are welcome! Please ensure:

1. Code is formatted with `cargo fmt`
2. All tests pass with `cargo test`
3. No clippy warnings with `cargo clippy`
4. Add tests for new features

## 📄 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- Windows Raw Input API documentation
- Rust community for excellent tooling
- Contributors and testers

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/tkykszk/KeyboardRemapperR/issues)
- **Discussions**: [GitHub Discussions](https://github.com/tkykszk/KeyboardRemapperR/discussions)

---

**Version**: v0.1.0-alpha1  
**Status**: Alpha (Core functionality complete, Phase 1-3)  
**Last Updated**: 2026-01-14
