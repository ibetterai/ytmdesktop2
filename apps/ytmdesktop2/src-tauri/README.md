# Tauri feasibility seam

This directory is a Phase 0 strangler-migration spike. It creates one unsigned, blank local Tauri host window and does not import, call, or replace any Electron code.

## Local commands

From the repository root, use the pinned pnpm and Rust toolchains:

```sh
pnpm install --frozen-lockfile
pnpm --filter ytmdesktop2 tauri:check
pnpm --filter ytmdesktop2 tauri:dev
```

`tauri:check` runs `cargo fmt --check` and `cargo test`; it compiles the Tauri shell and validates the Tauri configuration without opening a display window. `tauri:dev` runs the same isolated crate and opens the blank host window.

## Boundary and rollback

The Electron startup path, renderer routes, feature APIs, updater, signing, packaging, package identity, and release workflow are intentionally untouched. The only repository-level integration is the two `tauri:*` scripts in `apps/ytmdesktop2/package.json` and the non-GUI `tauri-feasibility` check in `.github/workflows/test.yml`.

To roll back the seam, delete `apps/ytmdesktop2/src-tauri`, remove those two scripts and the `tauri-feasibility` job. No Electron data, user state, credentials, packages, or release artifacts require migration or cleanup.
