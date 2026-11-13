# Third-Party Notices

Sayline is licensed under the Apache License, Version 2.0. See `LICENSE`.

This file lists the direct third-party components used by Sayline and their
licenses. Transitive dependencies are pinned in `impl/src-tauri/Cargo.lock` and
`impl/pnpm-lock.yaml`. Their license texts are distributed with the upstream
packages. Anyone distributing a binary build must include the full license
texts for all bundled dependencies, not only the direct ones listed here.

## Upstream reference

- **ZCode** — <https://github.com/zai-org/ZCode> — Apache-2.0.
  Sayline reimplements the ASR protocol and voice-input interaction patterns in
  Rust and React. It does not include the ZCode Electron runtime, application
  bundle, or any ZCode binary. See `NOTICE`.

## Rust dependencies

| Component | License |
| --- | --- |
| aes-gcm | Apache-2.0 OR MIT |
| arboard | MIT OR Apache-2.0 |
| base64 | MIT OR Apache-2.0 |
| chrono | MIT OR Apache-2.0 |
| dirs | MIT OR Apache-2.0 |
| futures-util | MIT OR Apache-2.0 |
| global-hotkey | Apache-2.0 OR MIT |
| keyring | MIT OR Apache-2.0 |
| reqwest | MIT OR Apache-2.0 |
| rusqlite | MIT |
| serde | MIT OR Apache-2.0 |
| serde_json | MIT OR Apache-2.0 |
| sha2 | MIT OR Apache-2.0 |
| symphonia | MPL-2.0 |
| tauri | Apache-2.0 OR MIT |
| tauri-build | Apache-2.0 OR MIT |
| tauri-plugin-dialog | Apache-2.0 OR MIT |
| tokio | MIT |
| tokio-tungstenite | MIT |
| tokio-util | MIT |
| tracing | MIT |
| tracing-subscriber | MIT |
| uuid | Apache-2.0 OR MIT |

`rusqlite` bundles SQLite, which is in the public domain.

## Frontend dependencies

| Component | License |
| --- | --- |
| react | MIT |
| react-dom | MIT |
| @tauri-apps/api | Apache-2.0 OR MIT |
| @tauri-apps/plugin-dialog | MIT OR Apache-2.0 |
| @tauri-apps/cli | Apache-2.0 OR MIT |
| @vitejs/plugin-react | MIT |
| typescript | Apache-2.0 |
| vite | MIT |

## System components

Sayline invokes or links against system components that are not bundled:

- webkit2gtk, GTK 3, libayatana-appindicator: dynamically linked.
- PulseAudio / PipeWire / ALSA tools (`parec`, `pw-record`, `arecord`): invoked
  as external commands.
- `xdotool`: invoked as an external command for X11 paste.

Their licenses and distribution terms are those of the host system.
