# Minimal macOS Tauri YouTube Music app

This directory builds a standalone, separately identified macOS Tauri app. It opens
`https://music.youtube.com` in Tauri's system WKWebView, permits only the YouTube
Music and Google Accounts sign-in origins for top-level navigation, and grants the
remote page no Tauri commands or plugins. Users sign in directly in this app's
separate Tauri profile; no Electron cookies or state are imported.

## Local commands

From the repository root, use the pinned pnpm and Rust toolchains:

```sh
rustup run stable cargo install tauri-cli --version 2.11.4 --locked
pnpm install --frozen-lockfile
pnpm --filter ytmdesktop2 tauri:check
pnpm --filter ytmdesktop2 tauri:dev
pnpm --filter ytmdesktop2 tauri:build:mac
```

`tauri:check` runs `cargo fmt --check` and locked Rust tests. `tauri:dev` opens
the native app for a local sign-in/playback smoke test. `tauri:build:mac` emits an
unsigned `.app` under `src-tauri/target/release/bundle/macos`; it contains a Tauri
WKWebView runtime, not Electron or Chromium.

## Boundary and rollback

The Electron startup path, renderer routes, feature APIs, updater, signing, packaging,
and default launch remain untouched. The Tauri app has its own bundle identifier and
does not take over Electron's package or protocol identity.

To roll back this app, remove the Tauri-specific files and `tauri:*` scripts. No
Electron data, user state, credentials, packages, or release artifacts require
migration or cleanup.
