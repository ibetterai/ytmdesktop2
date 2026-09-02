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

## Capability matrix

| Electron capability and source authority | Tauri contract, caller ACL, and platform | Typed behavior / errors | Data and path authority | Evidence, rollback, owner | Status |
| --- | --- | --- | --- | --- | --- |
| Main window state and chrome controls: `src/main/windows/windowManager.ts`, toolbar, `use-window-controls.ts` | `tauri_main_window_state` and `tauri_window_control`; injected caller window only; `capabilities/main.json` grants both only to `main`. macOS evidence target; no production UI caller. | `{ isMaximized }`; only `minimize` and `toggleMaximize`; native failures redact to `windowStateUnavailable` or `windowControlFailed`. | No user data, paths, URLs, arbitrary target windows, or caller-selected labels. | Rust serialization/error tests and `tests/tauri_config.rs` exact permission set. Revert commands, generated permissions, and ACL only. Owner: HALL-36. | implemented-and-verified |
| Title bar, layout, real main UI, default startup, and navigation | The configured `main` window has the empty `src-tauri/ui/index.html`; no invokes. HALL-83 permits a future isolated YouTube webview with the stated allowlist, not an implementation. | No Tauri UI/error contract is wired. | Future webview uses a fresh Tauri-only profile and never Electron cookies/session data. | Blank-shell config/HTML prove absence. Rollback remains deletion of the local shell. Owner: future cutover/webview slice. | unresolved |
| Settings storage, migration, and broadcasts: `src/main/trpc/routers/settings/service.ts`, `src/main/lib/store/createYmlStore.ts` | `tauri_plugin_bridge` → `settings.snapshot`, only `main` at exactly `tauri://localhost`, permission `allow-tauri-plugin-bridge`; no generic settings API or renderer caller. | V1 allows only `app.zoomFactor`; malformed keys → `invalidRequest`, unavailable state → `pluginFailed`; backing feasibility value is `1.0`. No write/event command is registered. | In-memory feasibility defaults only. HALL-83 requires distinct storage; migration must be future opt-in, copy-only, coexistent, and downgrade-safe. | Rust and TS bridge/settings tests; manifest/ACL exactness. Revert bridge/settings seam and permission only. Owner: HALL-38/HALL-39. | implemented-and-verified |
| Remote YouTube login, page-world bridge, and bundled plugins: `src/preload/`, `src/preload/ytmd-bridge.ts`, `src/renderer-plugins/youtube/` | Fixed `tauri_plugin_bridge` command only; `settings.snapshot` from `main`/`tauri://localhost`. HALL-83 permits future navigation only to the bounded YouTube/Google allowlist and no bundled third-party plugins initially. | Stable bridge errors: `unsupportedCommand`, `unauthorizedCaller`, `invalidRequest`, `pluginFailed`. No generic invoke, Node, eval, IPC, process, path, or arbitrary origin. | No Electron session/cookie/plugin data may be read, copied, or shared. | Rust dispatch/manifest tests and TS manifest/settings-client/disposable-host tests. Rollback removes bridge contract/permission/test host. Owner: HALL-39; each future webview/plugin needs review. | evidence-settled |
| Media controls / OS media session: `src/main/trpc/routers/mediaControl/service.ts` | `tauri_media_session_update`; `main` plus exact bundled Tauri origins; `allow-tauri-media-session-update`; macOS target; no caller UI. | Bounded V1 playback/track/timeline request and typed command event. Stable errors: `invalidRequest`, `unauthorizedCaller`, `unsupported`, `unavailable`, `operationFailed`. Backend intentionally returns `unavailable`. | No paths, URLs, native handles, or caller-selected targets. | Manifest, redaction/validation/origin and ACL tests; PR #21 independently reviewed at `6d0866b`. Revert module/ACL/permission. Owner: HALL-37. | implemented-and-verified |
| Global shortcuts: `src/main/trpc/routers/shortcut/service.ts` | `tauri_global_shortcut_set_registration`; `main` plus exact bundled Tauri origins; command-specific ACL; macOS target; no caller UI. | Only `{ contractVersion, enabled }` and fixed legacy mapping; typed trigger event; no accelerator/action input. Backend intentionally returns `unavailable`. | No paths, labels, process commands, or native handles. | Manifest, validation/origin/error and ACL tests; PR #22 independently reviewed at `93226fb`. Revert module/ACL/permission. Owner: HALL-37. | implemented-and-verified |
| Notifications and tray: `src/main/lib/appToast.ts`, `src/main/trpc/routers/tray/service.ts`, `src/main/handlers/trayState.ts` | `tauri_notification_present` and `tauri_tray_set_registration`, each permissioned only to `main` and exact bundled Tauri origins; macOS target; no caller UI. | Bounded toast `{ type, message }`, fixed registration toggle, typed tray event; rejects URL/icon/menu/path/window inputs. Both backends intentionally return `unavailable`. | No file paths, icons, menu IDs, or caller-selected windows. | Manifest, redaction/validation/origin and ACL tests; PR #23 independently reviewed at `9eee908`. Revert module/ACL/permissions. Owner: HALL-37. | implemented-and-verified |
| Chromecast discovery/session: `src/main/trpc/routers/chromecast/`, `src/main/chromecast/`, `src/preload/chromecast/` | No Tauri command, event, ACL, or webview. | No typed Tauri behavior/error. | No Tauri storage/network authority. | Electron-only evidence; separately reviewed optional adapter if reprioritized; no retirement dependency. Owner: future Backlog. | not supported by decision |
| Discord RPC: `src/main/lib/discord-rpc/`, `src/main/trpc/routers/discord/service.ts` | No Tauri command, event, ACL, or adapter. | No typed Tauri behavior/error. | No Tauri socket/process authority. | Electron-only evidence; separately reviewed optional adapter if reprioritized; no retirement dependency. Owner: future Backlog. | not supported by decision |
| Touch Bar: `src/main/trpc/routers/touchbar/service.ts` | No Tauri command, event, ACL, or adapter; optional after first release and only relevant to compatible macOS hardware. | No typed Tauri behavior/error. | No Tauri native-handle authority. | Electron-only evidence; separately reviewed optional adapter if reprioritized; no retirement dependency. Owner: future Backlog. | not supported by decision |
| Local API / Stream Deck / OBS: `src/main/api/`, `src/main/trpc/routers/api/service.ts`, `packages/streamdeck/` | No Tauri command, event, ACL, listener, or network permission. | No typed Tauri behavior/error. | No Tauri socket, token, or path authority. | Electron-only evidence; separately reviewed optional adapter if reprioritized; no retirement dependency. Owner: future Backlog. | not supported by decision |
| Quit, relaunch, hide-to-tray, provider cleanup, update install: `src/main/handlers/quitHandler.ts` | No Tauri command, event, lifecycle hook, or UI caller. | No Tauri behavior/error contract. | Electron remains authority for settings flush, provider destruction, tray policy, and updater install. | Source evidence shows this is more than a close button. A future lifecycle plan needs independent tests. Owner: future cutover slice. | unresolved |
| Package/protocol identity and deep links: `electron-builder.yml`, `src/main/trpc/routers/deeplink/service.ts`, `src/shared/protocol/ytmdProtocol.ts` | Distinct identifier `com.ibetterai.ytmdesktop2.tauri-spike`; HALL-83 keeps it and a separate test protocol until explicit cutover. | No Tauri deep-link handler. | No Electron identity, protocol registration, or user-data location is claimed. | `tauri.conf.json`/config tests verify the inactive shell. Revert the test seam; identity transfer needs separate authorization. Owner: HALL-40/HALL-83. | evidence-settled |
| Packaging, signing, publication, updater: `electron-builder.yml`, `.github/workflows/release.yml`, `src/main/trpc/routers/update/service.ts` | Bundle inactive. Test manifest: `macos`/`aarch64`, `0.0.0`, `test`, no endpoint. HALL-83 additionally requires arm64+x86_64 evidence, unsigned/non-published artifacts, updates disabled. | Verifier emits only `identityMismatch`, `unsupportedTarget`, `unsignedArtifact`; no updater contract. | Fixed test paths under `src-tauri/target`; no renderer input, credentials, signing material, endpoint, or real artifact write. | TS tests/README/config evidence; HALL-40 PR #24 independently reviewed/approved at `50f46c1`. Revert manifests/verifier only. Owner: HALL-40. | implemented-and-verified |

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
