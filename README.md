# M Reader

A lightweight, read-only Markdown previewer with auto-reload, recent files, and a clean, distraction-free desktop UI.

## Features
- Fast, native desktop app (Dioxus)
- Read-only Markdown preview with live reload
- Recent files list
- Multiple reader themes

## Requirements
- Rust toolchain (stable)
- `dx` (Dioxus CLI)

## Run (dev)
```bash
dx serve --desktop
```

## Build (release binary)
```bash
cargo build --release
```

## Bundle (macOS)
This project includes a helper script that ensures the app name and menu bar title are exactly "M Reader".

```bash
./scripts/bundle_macos.sh
```

Outputs:
- `dist/M Reader.app`
- `dist/M Reader_0.1.0_x64.dmg`

## Configuration
Bundle metadata and resources live in `Dioxus.toml`.

## License
TBD
