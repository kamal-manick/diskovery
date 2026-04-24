# Diskovery

A terminal-based disk space analyzer built in Rust. Scan any drive or folder, visualize file and folder sizes in an interactive tree view, identify space hogs, and delete items - all from your terminal.

![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-Windows-lightgrey)

## Features

- **Instant results** - top-level folders stream in as they finish scanning; no waiting for a full scan
- **Interactive tree view** - expand/collapse directories, navigate with keyboard
- **Visual size bars** - proportional bar next to each item shows relative size at a glance
- **Drive usage gauge** - total/used/free space shown at the top
- **Multi-select deletion** - select multiple files/folders, see total space freed before confirming
- **Permission-aware** - skipped paths (access denied) are reported, not crashed on

## Requirements

- [Rust 1.85+](https://rustup.rs)
- Windows 10 / 11 (primary target; Linux/macOS may work but are untested)
- A terminal with Unicode support (Windows Terminal recommended)

## Setup

```bash
git clone https://github.com/kamal-manick/diskovery.git
cd diskovery
```

## Build

Development build:

```bash
cargo build
```

Optimized release build (recommended for large drives):

```bash
cargo build --release
```

The release binary will be at `target/release/diskovery.exe`.

## Usage

```bash
# Scan your user profile directory (default)
cargo run

# Scan a specific path
cargo run -- "C:\Users\YourName\Downloads"

# Run the release build directly
.\target\release\diskovery.exe "C:\"
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor |
| `Enter` / `→` | Expand directory |
| `←` | Collapse directory / jump to parent |
| `Space` | Toggle select item |
| `A` | Select all visible items |
| `D` | Delete selected items (or item under cursor) |
| `R` | Rescan current directory |
| `Q` / `Esc` | Quit |
| `Page Up/Down` | Scroll half a page |

### Deletion flow

1. Select one or more items with `Space` (or leave nothing selected to target the cursor item)
2. Press `D` - a confirmation dialog shows all targets and the **total space that will be freed**
3. Use `Tab` / `←` / `→` to move between **Yes, Delete** and **Cancel**
4. Press `Enter` to confirm - items are deleted and a fresh scan starts automatically

## Project Structure

```
src/
  main.rs             - terminal setup/teardown, event loop, key handling
  app.rs              - application state, navigation, selection, delete logic
  scanner.rs          - parallel background scan with streaming results
  fs_tree.rs          - FsNode tree type, FlatItem projection, size sorting
  ui/
    mod.rs            - layout and render dispatcher
    drive_bar.rs      - drive usage gauge widget
    tree_view.rs      - scrollable tree list with size bars
    status_bar.rs     - key hints and scan progress
    confirm_dialog.rs - deletion confirmation modal
```

## License

MIT - see [LICENSE](LICENSE).
