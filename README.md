<p align="center">
  <img src="assets/icons/openwhatsapp.png" alt="OpenWhatsapp" width="96" height="96">
</p>

<h1 align="center">OpenWhatsapp</h1>

<p align="center">
  A native Linux desktop client for WhatsApp Web, built with Rust, GTK4, and WebKitGTK.
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021-f74c00?logo=rust&logoColor=white">
  <img alt="GTK4" src="https://img.shields.io/badge/GTK-4-4a86cf?logo=gtk&logoColor=white">
  <img alt="WebKitGTK" src="https://img.shields.io/badge/WebKitGTK-6.0-2f6db3?logo=webkit&logoColor=white">
  <img alt="Linux" src="https://img.shields.io/badge/Linux-native-fcc624?logo=linux&logoColor=111">
  <img alt="License" src="https://img.shields.io/badge/License-MIT-green">
</p>

## Overview

OpenWhatsapp is a lightweight, native WhatsApp Web wrapper for Linux. It is designed as an alternative to Electron-based clients: no Node runtime, no browser bundle, and no Chromium embed. The app uses GTK4 for the interface and WebKitGTK for rendering WhatsApp Web.

The project goal is a fast, stable desktop experience with native window behavior, tray integration, notifications, isolated account sessions, downloads, spellcheck, and customization support.

## Features

- Native GTK4 application window
- WebKitGTK 6 webview loading `https://web.whatsapp.com`
- System tray indicator with show, hide, and quit actions
- Native Linux notifications through `gio::Notification`
- Separate account storage paths for isolated sessions
- Account grid with unread badges
- Keyboard shortcuts for app actions and account switching
- Custom download routing
- Spellcheck integration through Enchant
- CSS and JavaScript customization pipeline
- Hyprland/Wayland redraw workarounds for WebKitGTK compositor freezes
- WhatsApp-compatible app and tray icon from `assets/icons/openwhatsapp.png`

## Current Status

OpenWhatsapp is under active development. Core app startup, tray, notifications, accounts, downloads, customizations, shortcuts, and WebKit setup are implemented.

Voice and video calls depend on WhatsApp Web support inside WebKitGTK. The app grants camera, microphone, device, and media permissions, but WhatsApp may still reject WebKitGTK as an unsupported browser depending on its server-side rollout and feature detection.

## Screenshots

Screenshots will be added once the first stable UI milestone is ready.

## Requirements

Build requirements:

- Rust stable
- GTK4 development files
- WebKitGTK 6 development files
- SQLite
- Enchant 2
- pkg-config

Runtime media and call dependencies:

```text
gstreamer1.0-plugins-base
gstreamer1.0-plugins-good
gstreamer1.0-plugins-bad
gstreamer1.0-libav
gstreamer1.0-pipewire
gstreamer1.0-pulseaudio
libwebrtc-audio-processing
xdg-desktop-portal
xdg-desktop-portal-hyprland
```

On Hyprland, PipeWire and `xdg-desktop-portal-hyprland` should be running for camera, microphone, and screen sharing paths to work correctly.

## Install Runtime Dependencies

Debian/Ubuntu-style systems:

```bash
./packaging/debian/install-runtime-deps.sh
```

Arch Linux package names differ. Install the equivalent GTK4, WebKitGTK 6, GStreamer, PipeWire, portal, SQLite, and Enchant packages from your distro repositories.

## Build

```bash
cargo build --release
```

Run:

```bash
target/release/openwhatsapp
```

Build with the optional GTK3/libappindicator tray backend:

```bash
cargo build --release --features appindicator-tray
```

## Logging

Use `RUST_LOG` for diagnostics:

```bash
RUST_LOG=openwhatsapp=debug target/release/openwhatsapp
```

This is useful when debugging WebKit load state, notification permission handling, downloads, and tray startup.

## Hyprland Notes

WebKitGTK can freeze on some Wayland compositors after workspace switches because repaint or damage events stop arriving. OpenWhatsapp applies several workarounds:

- periodic WebView redraw
- redraw on window focus and visibility changes
- redraw on GDK toplevel state changes
- WebKit compositor environment workarounds on Wayland/Hyprland

If the webview still freezes, try resizing the window once and run with debug logs to collect compositor behavior.

## Project Layout

```text
src/
  app/              GTK application bootstrap
  window/           decorations, fullscreen, scaling, geometry
  webview/          WebKitGTK setup, navigation, injection
  accounts/         account model, SQLite store, account grid
  tray/             system tray integration
  shortcuts/        keyboard shortcuts
  notifications/    native notification bridge
  spellcheck/       Enchant language support
  downloads/        download destination handling
  customizations/   CSS/JS import, store, editor, injection
  settings/         settings dialog panels
  ipc/              broadcast event bus
  config/           TOML config schema and migration
  theme/            theme preference and watcher
  drag_drop/        file drop support
  utils/            logging and filesystem paths
```

## Development Checks

Before submitting changes:

```bash
cargo fmt
cargo build
cargo clippy -- -D warnings
cargo test
```

The codebase follows these rules:

- no `unwrap()` outside tests
- GTK and GLib work stays on the main thread
- async work runs on Tokio and returns to GTK through the main context
- modules communicate through the central IPC event bus
- public APIs are exposed through module boundaries only

## Roadmap

- Multi-account UX polish
- Settings dialog completion
- Better customization editor workflow
- Packaged releases for major Linux distributions
- Desktop portal integration hardening
- Performance pass for cold start and idle memory

## License

MIT

## Disclaimer

OpenWhatsapp is an unofficial client and is not affiliated with WhatsApp, Meta, GTK, or WebKitGTK. WhatsApp and related trademarks belong to their owners.
