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
as such; it is never implied by the blank UI or plugin bridge.

## Capability matrix

| Electron capability and source authority | Tauri contract, caller ACL, and platform | Typed behavior / errors | Data and path authority | Evidence, rollback, owner | Status |
| --- | --- | --- | --- | --- | --- |
| Main window state and chrome controls: `src/main/windows/windowManager.ts`, toolbar, `use-window-controls.ts` | `tauri_main_window_state` and `tauri_window_control`; injected caller window only; `capabilities/main.json` grants both only to `main`. macOS evidence target; no production UI caller. | `{ isMaximized }`; only `minimize` and `toggleMaximize`; native failures redact to `windowStateUnavailable` or `windowControlFailed`. | No user data, paths, URLs, arbitrary target windows, or caller-selected labels. | Rust serialization/error tests and `tests/tauri_config.rs` exact permission set. Revert commands, generated permissions, and ACL only. Owner: HALL-36. | implemented-and-verified |
| Auxiliary-window state, always-on-top, and dialog responses: `src/main/trpc/routers/window/router.ts`, `src/main/windows/` | No Tauri command/event/ACL for `state`, `mainState`, `stayOnTop`, dialog response, or subscriptions beyond the narrow main-window controls above. macOS evidence target only. | No Tauri behavior/error contract. | Electron owns sender-specific window state and dialog IDs; no Tauri window label or dialog target is accepted. | Exact ACL list proves omission; no Tauri behavioral test exists. Future window/lifecycle slice must add typed caller-scoped contracts. Rollback: no Tauri feature to remove. Owner: future cutover slice. | unresolved |
| App process, auxiliary windows, file/log opening, restart, and single-instance behavior: `src/main/trpc/routers/app/router.ts`, `app/service.ts` | No Tauri command/event/ACL for `openFile`, `openLogsFolder`, subwindows, settings/restart dialogs, or process lifecycle. | No Tauri behavior/error contract. | Electron owns shell/file paths, log location, BrowserWindow map, and single-instance lock. Tauri has no filesystem or shell permission. | Source inspection plus exact ACL check establish absence; Electron tests do not prove Tauri parity. Future slice needs typed file/path policy and lifecycle tests. Rollback: no Tauri feature to remove. Owner: future cutover slice. | unresolved |
| Title bar, layout, real main UI, default startup, and navigation | The configured `main` window has the empty `src-tauri/ui/index.html`; no invokes. HALL-83 permits a future isolated YouTube webview with the stated allowlist, not an implementation. | No Tauri UI/error contract is wired. | Future webview uses a fresh Tauri-only profile and never Electron cookies/session data. | Blank-shell config/HTML prove absence. Rollback remains deletion of the local shell. Owner: future cutover/webview slice. | unresolved |
| Player track/like/playback/seek/volume/state: `src/main/trpc/routers/track/router.ts`, `track/service.ts` | No Tauri player command, event, UI caller, or ACL. The media-session seam reports a bounded OS-media-state contract only; it cannot control the YouTube player. | No Tauri playback, queue, like/dislike, seek, volume, or state error contract. | Electron webContents/YouTube page remain authority; no Tauri cookie, page-world, or remote-webview authority exists. | Router/source inspection and exact ACL set establish absence. Future isolated-webview/player contract needs typed requests/events and tests. Rollback: no Tauri feature to remove. Owner: future webview/player slice. | unresolved |
| YouTube navigation, watch/playlist/channel URLs, and queue management: `src/main/trpc/routers/navigation/router.ts`, `navigation/service.ts` | No Tauri navigation/queue command, event, or ACL. HALL-83 supplies only the future origin policy: `music.youtube.com` plus bounded Google sign-in; it is not a navigation implementation. | No Tauri URL validation, queue result, or stable error contract. | No Tauri session, arbitrary URL, or deep-link authority. Electron validates/opens current YouTube and `ytmd` routes. | Router/source inspection and blank UI establish absence. Future webview slice needs allowlist/typed URL tests. Rollback: no Tauri feature to remove. Owner: future webview/navigation slice. | unresolved |
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
| Package/protocol identity and deep links: `electron-builder.yml`, `src/main/trpc/routers/deeplink/service.ts`, `src/shared/protocol/ytmdProtocol.ts` | Distinct identifier `com.ibetterai.ytmdesktop2.tauri-spike`; HALL-83 keeps it and a separate test protocol until explicit cutover. | No Tauri deep-link handler. | No Electron identity, protocol registration, or user-data location is claimed. | `tauri.conf.json`/config tests verify the inactive shell. Revert the test seam; identity transfer needs separate authorization. Owner: HALL-40/HALL-83. | evidence-settled |
| Electron runtime updater: `src/main/trpc/routers/update/router.ts`, `update/service.ts` | No Tauri update command, event, ACL, endpoint, or UI. The test artifact contract below is explicitly not an updater. | No Tauri check/download/install/progress/error contract. | Electron owns endpoint/channel, updater cache, download/install lifecycle, and release credentials. | Router/source inspection and HALL-83 no-endpoint/no-credential boundary establish absence. Future updater work requires separate authorization. Rollback: no Tauri feature to remove. Owner: future cutover/release slice. | unresolved |
| Non-production release artifact only: `src-tauri/release-artifact-manifest.json`, `src/tauri-release-artifact.ts` | Bundle inactive. Test manifest: `macos`/`aarch64`, `0.0.0`, `test`, no endpoint. HALL-83 additionally requires arm64+x86_64 evidence, unsigned/non-published artifacts, updates disabled. | Verifier emits only `identityMismatch`, `unsupportedTarget`, `unsignedArtifact`; no updater contract. | Fixed test paths under `src-tauri/target`; no renderer input, credentials, signing material, endpoint, or real artifact write. | TS tests/README/config evidence; HALL-40 PR #24 independently reviewed/approved at `50f46c1`. Revert manifests/verifier only. Owner: HALL-40. | implemented-and-verified |

## Verification evidence

- `pnpm test` runs the JavaScript/Vitest suite, including bridge and staged-artifact tests.
- `pnpm --filter ytmdesktop2 tauri:check` runs `cargo fmt --check` and locked Rust tests; `src-tauri/tests/tauri_config.rs` asserts the exact capability set and blank-window configuration.
- `.github/workflows/test.yml` runs the same non-GUI Tauri feasibility check.
- The only safe desktop smoke is `pnpm --filter ytmdesktop2 tauri:dev` and visual confirmation of the blank local window. It needs no production credentials or real account, but cannot show navigation, login, media, tray, shortcuts, notifications, data compatibility, packaging, updates, or parity because those paths are absent/inactive.

## Electron-retirement decision

**Do not retire Electron and do not change default launch.** Tauri is a useful,
independently-tested contract/evidence seam, not a replacement application.
Electron remains sole authority for real UI/navigation, durable settings,
lifecycle/quit, deep links, production identity/release/updater, and every
optional post-first-release integration.

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
