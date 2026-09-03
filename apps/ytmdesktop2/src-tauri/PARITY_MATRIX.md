# Tauri parity matrix and Electron-retirement checklist

## Scope and status vocabulary

This is an evidence record for Phase 0 at `a81a76a`. It is not a claim that the
Electron application has been ported. An **implemented-and-verified** row proves
only the listed inactive Tauri contract; it does not make the Electron feature
available through Tauri.

| Status | Meaning |
| --- | --- |
| implemented-and-verified | Source, narrow contract tests, and ACL exact-set tests exist; the backend can still deliberately return `unavailable`. |
| evidence-settled | Source or an accepted decision establishes the current boundary. |
| not supported by decision | HALL-51/HALL-83 explicitly excludes it from the first Tauri release. |
| unresolved | Future implementation or cutover work is needed; parity must not be inferred. |

**Controlling decisions.** HALL-51 sets macOS as the first-release platform;
Chromecast/discovery, Discord RPC, Touch Bar, and Local API are optional after
that release. HALL-83 sets non-production defaults: one fresh Tauri-only
profile, `https://music.youtube.com/*` plus the bounded
`https://accounts.google.com/*` sign-in redirect, no bundled third-party
plugins, a separate test package/protocol, macOS arm64 and x86_64 evidence, and
unsigned endpoint-free artifacts with updates disabled. Neither decision
authorizes a production cutover.

### Electron router-registry coverage

`src/main/trpc/router.ts` is the coverage inventory, not a Tauri contract.
Each registered router maps to an explicit row below: `track` → player state and
commands; `settings` → settings; `app` → process/auxiliary windows; `api` and
`auth` → local API/pairing; `update` → runtime updater and test artifacts;
`navigation` → navigation/queue; `tray` and `trayView` → notification/tray and
tray view; `themes` → themes; `lastfm` → Last.fm; `chromecast`, `window`, and
`discord` → their named rows. An empty or restricted Tauri side is classified
as such; it is never implied by a webview or plugin bridge.

## Current macOS runtime profile (HALL-85)

The standalone Tauri app opens `https://music.youtube.com/` in a native
WKWebView. Top-level navigation is limited to that exact host and
`https://accounts.google.com/` for sign-in. The remote webview receives no Tauri
permissions, commands, plugins, filesystem access, shell access, or Electron
cookies/state. The legacy command seams below remain source-level evidence only:
the empty `capabilities/main.json` permission set leaves them unavailable at
runtime. Electron remains the default app and its sources are unchanged.

## Capability matrix

| Electron capability and source authority | Tauri contract, caller ACL, and platform | Typed behavior / errors | Data and path authority | Evidence, rollback, owner | Status |
| --- | --- | --- | --- | --- | --- |
| Main window state and chrome controls: `src/main/windows/windowManager.ts`, toolbar, `use-window-controls.ts` | `tauri_main_window_state` and `tauri_window_control`; injected caller window only; `capabilities/main.json` grants both only to `main`. macOS evidence target; no production UI caller. | `{ isMaximized }`; only `minimize` and `toggleMaximize`; native failures redact to `windowStateUnavailable` or `windowControlFailed`. | No user data, paths, URLs, arbitrary target windows, or caller-selected labels. | Rust serialization/error tests and `tests/tauri_config.rs` exact permission set. Revert commands, generated permissions, and ACL only. Owner: HALL-36. | implemented-and-verified |
| Auxiliary-window state, always-on-top, and dialog responses: `src/main/trpc/routers/window/router.ts`, `src/main/windows/` | No Tauri command/event/ACL for `state`, `mainState`, `stayOnTop`, dialog response, or subscriptions beyond the narrow main-window controls above. macOS evidence target only. | No Tauri behavior/error contract. | Electron owns sender-specific window state and dialog IDs; no Tauri window label or dialog target is accepted. | Exact ACL list proves omission; no Tauri behavioral test exists. Future window/lifecycle slice must add typed caller-scoped contracts. Rollback: no Tauri feature to remove. Owner: future cutover slice. | unresolved |
| App process, auxiliary windows, file/log opening, restart, and single-instance behavior: `src/main/trpc/routers/app/router.ts`, `app/service.ts` | No Tauri command/event/ACL for `openFile`, `openLogsFolder`, subwindows, settings/restart dialogs, or process lifecycle. | No Tauri behavior/error contract. | Electron owns shell/file paths, log location, BrowserWindow map, and single-instance lock. Tauri has no filesystem or shell permission. | Source inspection plus exact ACL check establish absence; Electron tests do not prove Tauri parity. Future slice needs typed file/path policy and lifecycle tests. Rollback: no Tauri feature to remove. Owner: future cutover slice. | unresolved |
| Title bar, layout, real main UI, default startup, and navigation | `main.rs` creates one native `main` WKWebView at YouTube Music; `tauri.conf.json` declares no local window. Electron remains the default launch. | The native window has no Tauri command surface; remote web content supplies its own UI. | Tauri uses its own bundle identifier and profile; Electron cookies/session data are never imported. | Rust navigation/config tests; rollback removes the isolated Tauri app without touching Electron. Owner: HALL-85. | implemented-and-verified |
| Player track/like/playback/seek/volume/state: `src/main/trpc/routers/track/router.ts`, `track/service.ts` | Playback is performed by YouTube Music in the WKWebView; no Tauri player command, event, or native media-control ACL is exposed. | No typed native playback, queue, like/dislike, seek, volume, or state contract. | The remote YouTube Music session in Tauri is independent from Electron webContents and cookies. | Remote playback is intentionally web-content-owned. Future native media control remains separate. Owner: future slice. | unresolved |
| YouTube navigation, watch/playlist/channel URLs, and queue management: `src/main/trpc/routers/navigation/router.ts`, `navigation/service.ts` | The Tauri navigation callback accepts HTTPS only at exact `music.youtube.com` and `accounts.google.com` hosts. There is no native queue/deep-link command. | Disallowed top-level navigations return `false` to the webview runtime; no external URL input is accepted by Rust. | No arbitrary URL, Electron session, or `ytmd` deep-link authority. | Unit tests cover allowed hosts, scheme enforcement, and lookalike rejection. Rollback removes the callback/window. Owner: HALL-85. | implemented-and-verified |
| Settings storage, migration, and broadcasts: `src/main/trpc/routers/settings/service.ts`, `src/main/lib/store/createYmlStore.ts` | `tauri_plugin_bridge` → `settings.snapshot`, only `main` at exactly `tauri://localhost`, permission `allow-tauri-plugin-bridge`; no generic settings API or renderer caller. | V1 allows only `app.zoomFactor`; malformed keys → `invalidRequest`, unavailable state → `pluginFailed`; backing feasibility value is `1.0`. No write/event command is registered. | In-memory feasibility defaults only. HALL-83 requires distinct storage; migration must be future opt-in, copy-only, coexistent, and downgrade-safe. | Rust and TS bridge/settings tests; manifest/ACL exactness. Revert bridge/settings seam and permission only. Owner: HALL-38/HALL-39. | implemented-and-verified |
| Themes, CSS injection, custom theme files, and file watching: `src/main/trpc/routers/themes/router.ts`, `themes/service.ts`, `themes/compiler.ts` | No Tauri theme list/reload command, event, UI caller, filesystem ACL, or CSS injection seam. | No Tauri compile/watch/injection behavior or error contract. | Electron reads/writes/watches custom CSS/SCSS (including Documents default path) and injects into Electron webContents. Tauri has no file/path authority. | Router/source inspection and exact ACL set establish absence. Future themes work needs separate file consent/path policy and renderer/webview tests. Rollback: no Tauri feature to remove. Owner: future themes slice. | unresolved |
| Last.fm account authorization, session state, now-playing, and scrobbling: `src/main/trpc/routers/lastfm/router.ts`, `lastfm/service.ts` | No Tauri command, event, UI caller, ACL, or network adapter. | No Tauri auth/scrobble status or error contract. | Electron owns the Last.fm auth window, secure session/settings, and external account traffic. HALL-83 authorizes neither credentials nor real-account access for evidence. | Router/source inspection and exact ACL set establish absence. A future credential/network design needs separate authorization and tests. Rollback: no Tauri feature to remove. Owner: future integration slice. | unresolved |
| Remote YouTube login, page-world bridge, and bundled plugins: `src/preload/`, `src/preload/ytmd-bridge.ts`, `src/renderer-plugins/youtube/` | Fixed `tauri_plugin_bridge` command only; `settings.snapshot` from `main`/`tauri://localhost`. HALL-83 permits future navigation only to the bounded YouTube/Google allowlist and no bundled third-party plugins initially. | Stable bridge errors: `unsupportedCommand`, `unauthorizedCaller`, `invalidRequest`, `pluginFailed`. No generic invoke, Node, eval, IPC, process, path, or arbitrary origin. | No Electron session/cookie/plugin data may be read, copied, or shared. | Rust dispatch/manifest tests and TS manifest/settings-client/disposable-host tests. Rollback removes bridge contract/permission/test host. Owner: HALL-39; each future webview/plugin needs review. | evidence-settled |
| Media controls / OS media session: `src/main/trpc/routers/mediaControl/service.ts` | `tauri_media_session_update`; `main` plus exact bundled Tauri origins; `allow-tauri-media-session-update`; macOS target; no caller UI. | Bounded V1 playback/track/timeline request and typed command event. Stable errors: `invalidRequest`, `unauthorizedCaller`, `unsupported`, `unavailable`, `operationFailed`. Backend intentionally returns `unavailable`. | No paths, URLs, native handles, or caller-selected targets. | Manifest, redaction/validation/origin and ACL tests; PR #21 independently reviewed at `6d0866b`. Revert module/ACL/permission. Owner: HALL-37. | implemented-and-verified |
| Global shortcuts: `src/main/trpc/routers/shortcut/service.ts` | `tauri_global_shortcut_set_registration`; `main` plus exact bundled Tauri origins; command-specific ACL; macOS target; no caller UI. | Only `{ contractVersion, enabled }` and fixed legacy mapping; typed trigger event; no accelerator/action input. Backend intentionally returns `unavailable`. | No paths, labels, process commands, or native handles. | Manifest, validation/origin/error and ACL tests; PR #22 independently reviewed at `93226fb`. Revert module/ACL/permission. Owner: HALL-37. | implemented-and-verified |
| Notifications and tray: `src/main/lib/appToast.ts`, `src/main/trpc/routers/tray/service.ts`, `src/main/handlers/trayState.ts` | `tauri_notification_present` and `tauri_tray_set_registration`, each permissioned only to `main` and exact bundled Tauri origins; macOS target; no caller UI. | Bounded toast `{ type, message }`, fixed registration toggle, typed tray event; rejects URL/icon/menu/path/window inputs. Both backends intentionally return `unavailable`. | No file paths, icons, menu IDs, or caller-selected windows. | Manifest, redaction/validation/origin and ACL tests; PR #23 independently reviewed at `9eee908`. Revert module/ACL/permissions. Owner: HALL-37. | implemented-and-verified |
| Tray-view popover, pinning, and return-to-main actions: `src/main/trpc/routers/trayView/router.ts`, `trayView/service.ts` | No Tauri command/event/UI/ACL for popover visibility, pinned state, or `openMain`. The inactive tray-registration seam above is not a tray view. | No Tauri tray-view state/result/error contract. | Electron owns tray-view window state and settings-backed pin preference; Tauri has no persistent tray-view state. | Router/source inspection and exact ACL set establish absence. Future tray-view slice needs scoped state/events and lifecycle tests. Rollback: no Tauri feature to remove. Owner: future tray-view slice. | unresolved |
| Chromecast discovery/session: `src/main/trpc/routers/chromecast/`, `src/main/chromecast/`, `src/preload/chromecast/` | No Tauri command, event, ACL, or webview. | No typed Tauri behavior/error. | No Tauri storage/network authority. | Electron-only evidence; separately reviewed optional adapter if reprioritized; no retirement dependency. Owner: future Backlog. | not supported by decision |
| Discord RPC: `src/main/lib/discord-rpc/`, `src/main/trpc/routers/discord/service.ts` | No Tauri command, event, ACL, or adapter. | No typed Tauri behavior/error. | No Tauri socket/process authority. | Electron-only evidence; separately reviewed optional adapter if reprioritized; no retirement dependency. Owner: future Backlog. | not supported by decision |
| Touch Bar: `src/main/trpc/routers/touchbar/service.ts` | No Tauri command, event, ACL, or adapter; optional after first release and only relevant to compatible macOS hardware. | No typed Tauri behavior/error. | No Tauri native-handle authority. | Electron-only evidence; separately reviewed optional adapter if reprioritized; no retirement dependency. Owner: future Backlog. | not supported by decision |
| Local API status/listener plus Stream Deck/OBS: `src/main/api/`, `src/main/trpc/routers/api/router.ts`, `packages/streamdeck/` | No Tauri command, event, ACL, listener, or network permission. | No typed Tauri behavior/error. | No Tauri socket, token, or path authority. | Electron-only evidence; separately reviewed optional adapter if reprioritized; no retirement dependency. Owner: future Backlog. | not supported by decision |
| Local API pairing, client tokens, and approval UI: `src/main/trpc/routers/auth/router.ts`, `auth/service.ts`, `src/main/auth/` | No Tauri command, event, ACL, secret-store, or pairing UI. This is part of the optional-after-first-release Local API capability, not an implicit settings/webview feature. | No Tauri token, approval, revoke, or stable error contract. | Electron encrypts and persists API client tokens; no Tauri secret or credential authority exists. | Router/source inspection and exact ACL set establish absence. A separately reviewed optional adapter must define secret custody and revocation tests. Rollback: no Tauri feature to remove. Owner: future Backlog. | not supported by decision |
| Quit, relaunch, hide-to-tray, provider cleanup, update install: `src/main/handlers/quitHandler.ts` | No Tauri command, event, lifecycle hook, or UI caller. | No Tauri behavior/error contract. | Electron remains authority for settings flush, provider destruction, tray policy, and updater install. | Source evidence shows this is more than a close button. A future lifecycle plan needs independent tests. Owner: future cutover slice. | unresolved |
| Package/protocol identity and deep links: `electron-builder.yml`, `src/main/trpc/routers/deeplink/service.ts`, `src/shared/protocol/ytmdProtocol.ts` | Distinct identifier `com.ibetterai.ytmdesktop2.tauri`; no Tauri deep-link handler. | No Tauri deep-link behavior. | No Electron package identity, protocol registration, or user-data location is claimed. | `tauri.conf.json`/config tests verify the separate app identifier. Identity transfer still needs explicit authorization. Owner: HALL-85. | implemented-and-verified |
| Electron runtime updater: `src/main/trpc/routers/update/router.ts`, `update/service.ts` | No Tauri update command, event, ACL, endpoint, or UI. The test artifact contract below is explicitly not an updater. | No Tauri check/download/install/progress/error contract. | Electron owns endpoint/channel, updater cache, download/install lifecycle, and release credentials. | Router/source inspection and HALL-83 no-endpoint/no-credential boundary establish absence. Future updater work requires separate authorization. Rollback: no Tauri feature to remove. Owner: future cutover/release slice. | unresolved |
| Non-production release artifact only: `src-tauri/release-artifact-manifest.json`, `src/tauri-release-artifact.ts` | The legacy test-artifact verifier remains endpoint-free. HALL-85 additionally enables a separate, unsigned macOS `.app` bundle with updates disabled. | The verifier emits only `identityMismatch`, `unsupportedTarget`, `unsignedArtifact`; neither it nor the bundle exposes an updater. | Fixed test paths under `src-tauri/target`; no renderer input, credentials, signing material, endpoint, or publishing path. | TS verifier tests plus Tauri config/build verification. Revert manifests/verifier or the isolated bundle independently. Owner: HALL-40/HALL-85. | implemented-and-verified |

## Verification evidence

- `pnpm test` runs the JavaScript/Vitest suite, including bridge and staged-artifact tests.
- `pnpm --filter ytmdesktop2 tauri:check` runs `cargo fmt --check` and locked Rust tests; `src-tauri/tests/tauri_config.rs` asserts the empty native permission set and bundled macOS configuration.
- `.github/workflows/test.yml` runs the same non-GUI Tauri check.
- `pnpm --filter ytmdesktop2 tauri:dev` is the desktop smoke: it loads YouTube Music in WKWebView, where a user can sign in and start playback. It does not prove native media controls, tray, shortcuts, data compatibility, updater, or general Electron parity.

## Electron-retirement decision

**Do not retire Electron and do not change default launch.** Tauri is a useful,
independently-tested contract/evidence seam, not a replacement application.
Electron remains the default application and sole authority for its durable
settings, lifecycle/quit, deep links, production identity/release/updater, and
every optional post-first-release integration.

### Required future cutover authorization and evidence

Before Electron removal can be considered, a separate cutover issue must:

1. Authorize identity/protocol transfer, default launch, credentials,
   signing/notarization, updater-key custody, endpoint/channel, and publishing;
   then test coexistence and downgrade/install behavior.
2. Implement and independently verify a real Tauri UI under the constrained
   YouTube navigation/session policy, plus each capability selected for release.
3. Implement a durable, opt-in, copy-only migration with downgrade proof; never
   infer permission to read, move, or delete Electron user data.
4. Implement/test lifecycle semantics, deep links, and production
   release/update behavior.
5. Produce clean macOS arm64 and x86_64 evidence at the exact reviewed head and
   preserve the protected review pipeline without production publication.
6. Obtain cutover approval, then plan a reversible staged retirement. Until
   then, rollback is only removal/reversion of Tauri-local inactive seams;
   Electron data, identity, artifacts, and release path remain untouched.
