# Recenter Mouse (Rust)

A blazing-fast, ultra-lightweight Windows utility that instantly recenters your mouse cursor (and active window) to the primary display. Perfect for multi-monitor setups when your cursor is lost or active windows need to be snapped back.

Rewrite of the original Python/Conda [bookamp/Recenter-Mouse](https://github.com/bookamp/Recenter-Mouse), optimized for zero background CPU usage, minimal memory footprint, and native Windows integration.

---

## ⚡ Performance Comparison

| Metric | Original (Python / Conda) | **Rust Native Rewrite** | Improvement |
| :--- | :--- | :--- | :--- |
| **Idle Memory (RAM)** | ~60 MB – 150 MB+ | **~1 MB – 3 MB** | **~98% reduction** |
| **Idle CPU Usage** | Periodic hooks/polling | **0.0%** (Win32 event-driven) | Zero CPU impact |
| **Executable Size** | Hundreds of MBs (Python + packages) | **~312 KB** (single `.exe`) | **> 99% smaller** |
| **External Dependencies** | Python 3.11, Conda, PyAutoGUI, PyWin32, Pynput | **None** (pure native Win32 API) | Self-contained |
| **Startup Time** | 2 – 5 seconds (conda & scripts) | **< 5 milliseconds** | Instantaneous |
| **Process Management** | Batch + WMIC loop + PowerShell | **Windows Mutex** (atomic single instance) | Clean & reliable |

---

## ✨ Features

- **Instant Cursor Recentering**: Direct Win32 API cursor repositioning to the exact pixel center of your primary monitor.
- **Active Window Relocation**: Restores/unmaximizes the foreground window and places it on your main display (can be toggled on/off).
- **Per-Monitor V2 DPI Aware**: Accurate coordinate calculations across mixed-DPI, multi-monitor display layouts.
- **Native Settings & Key-Binder GUI**: A native dialog window to easily customize hotkey modifiers (`Ctrl`, `Alt`, `Shift`, `Win`), choose trigger keys (`A`-`Z`, `F1`-`F12`, `Space`, etc.), and toggle behaviors.
- **Interactive System Tray Icon**:
  - **Left-Click / Double-Click**: Instantly triggers recentering.
  - **Right-Click Menu**:
    - *Recenter Now (Ctrl + Alt + C)*
    - *Move Active Window* (Live checkmark toggle)
    - *Settings & Key Bindings...*
    - *Start with Windows* (Live checkmark toggle)
    - *Exit*
- **Single-Instance Protection**: Prevents duplicate instances cleanly with a system named Mutex.
- **No Console Flash**: Compiles with `#![windows_subsystem = "windows"]` for completely silent startup.

---

## 📦 Installation & Download

Pre-built binaries are available on the [Releases](https://github.com/bookamp/Recenter-Mouse-Rust/releases) page:

1. **Windows Installer (`.msi`)**:
   - Standard Windows setup wizard.
   - Installs to `Program Files\Recenter Mouse`.
   - Adds Start Menu shortcuts with embedded high-resolution icon.
   - Clean uninstallation via Windows *Installed Apps / Add or Remove Programs*.
   - Supports silent installation (`msiexec /i recenter-mouse-v1.0.0-x64.msi /qn`).

2. **Portable Version (`.zip`)**:
   - Zero installation required.
   - Extract anywhere (e.g. `C:\Tools\RecenterMouse`) and double-click `recenter-mouse.exe`.
   - Settings persist automatically in `%APPDATA%\RecenterMouse\config.json`.

---

## ⌨️ Default Shortcut

Press: **`Ctrl + Alt + C`**

> You can change this to any combination you prefer via the **Settings & Key Bindings** interface or directly in the configuration file.

---

## 🖥️ Command Line Options

```text
recenter-mouse.exe [OPTIONS]

Options:
  --once, -o       Recenter cursor and active window once, then immediately exit
  --settings, -s   Open the Settings & Key Bindings configuration dialog
  --headless       Run the hotkey listener in background without a tray icon
  --help, -h       Display help dialog
```

### Automation & Stream Deck / Macros
Because `--once` executes recentering and exits immediately, you can bind `recenter-mouse.exe --once` to Stream Deck buttons, gaming mouse keys, or custom scripts without keeping a daemon running if preferred!

---

## ⚙️ Configuration

Settings are automatically saved in JSON format at:
```text
%APPDATA%\RecenterMouse\config.json
```

Example configuration:
```json
{
  "ctrl": true,
  "alt": true,
  "shift": false,
  "win": false,
  "key": "C",
  "move_window": true,
  "start_with_windows": false
}
```

---

## 🔨 Building from Source

### Prerequisites
- Windows 10 or 11
- [Rust & Cargo](https://rustup.rs/) (stable channel)

### Build Release Binary
```powershell
# Clone or navigate to the project directory
cd e:\Projects\recenter-mouse-rust

# Run tests
cargo test

# Build optimized release binary
cargo build --release
```

The resulting standalone executable will be located at:
```text
target\release\recenter-mouse.exe
```
*(~312 KB standalone binary, no installer or runtime required)*

---

## 📄 License

MIT License. Designed and optimized for Windows.
