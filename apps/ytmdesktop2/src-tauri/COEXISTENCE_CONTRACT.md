# Optional macOS Tauri pilot coexistence contract

This contract governs the next **opt-in, non-production** macOS Tauri pilot.
It is a planning boundary, not approval to launch a pilot, transfer identity,
or cut over the desktop application.

## Evidence snapshot and non-approval

The repository's merged history supplies these relevant, reviewable baselines:

- `a81a76a` (HALL-40 / PR #24) defines the inactive, unsigned test-artifact
  contract with its distinct Tauri identity.
- `f69853a` (HALL-94 / PR #30) adds the redacted manual-acceptance protocol.
- `a0bb86d` (PR #31) validates the committed Tauri source configuration.

`bf3950b` is the source commit for HALL-85's minimal macOS Tauri music-app
slice. Its associated PR #28 and HALL-85 acceptance must be treated as
unresolved until the approved, exact-head acceptance evidence is recorded; the
presence of source in repository history is not acceptance. HALL-93's bundle
inspection and HALL-95's authorized smoke are likewise unresolved dependencies.
This document neither advances nor changes the state of any of those issues.

## Coexistence invariants

Electron remains the supported and default application:

- Root `dev`, `start`, `build`, and every `release*` command delegate to the
  Electron application. `apps/ytmdesktop2/package.json` keeps Electron as its
  `main` entry and maps `dev`, `start`, build, and packaging commands to
  `electron-vite` or `electron-builder`.
- `src/main/main.ts` starts Electron, creates the root Electron window, and
  registers the app and thumbnail protocols. `electron-builder.yml` retains
  the Electron package identity and the registered `ytmd` protocol.
- Electron remains the owner of its `userData` storage, including settings,
  caches, logs, and migration paths. A pilot must use a fresh Tauri-only
  profile: it must not read, copy, migrate, move, delete, or clean up Electron
  data, cookies, credentials, tokens, or local state.
- The Tauri feasibility configuration has the separate
  `com.ibetterai.ytmdesktop2.tauri-spike` test identity, inactive bundling, and
  no Electron deep-link or updater authority. It must not claim the Electron
  package identity, `ytmd` protocol, signing identity, update channel, or
  release artifacts.

The only permitted pilot entry point is an explicitly invoked Tauri command or
an independently reviewed, separately identified test artifact. It must never
be invoked by the default Electron launcher, Electron packaging/release
workflow, login item, deep-link handler, or updater. Starting Electron remains
the normal path even while a tester has opted into the pilot.

`pnpm --filter ytmdesktop2 electron:default-launch-check` reads only the
committed root and desktop package manifests. It verifies the Electron entry,
ordinary launch/build scripts, and release/package commands remain on the
Electron path. Tauri commands, if present, remain explicitly named `tauri:*`
opt-ins; the check permits no such commands and does not imply a pilot exists,
is accepted, or is ready to launch.

## Entry gates and observable outcomes

All gates are cumulative. A missing gate is a stop condition, not a reason to
infer approval.

| Gate | Evidence | Lane | Failure signal |
| --- | --- | --- | --- |
| Baseline acceptance | HALL-85 has an exact-head independent acceptance decision; HALL-95 supplies only the redacted, authorized manual observations required for it. | Tessa macOS lane for HALL-95; Clara decision | Missing, failed, or non-matching evidence. |
| Artifact isolation | HALL-93's deterministic check verifies the reviewed test bundle has its separate identifier and no Electron/Chromium payload. | Repository-only | Check is missing, skipped, or fails. |
| Default preservation | Source/config check shows Electron `dev`/`start`/release remain unchanged and no default path calls Tauri. | Repository-only | Any launcher, package, protocol, updater, identity, or release-path overlap. |
| Pilot review | A small, Tauri-local change has independent review, exact-head checks, and an explicit opt-in invocation. | Repository-only | Change touches Electron authority or lacks an opt-in boundary. |
| Desktop observation | The exact reviewed test artifact launches and exhibits only the test's stated behavior under the redacted protocol. | Tessa macOS lane | Launch, origin, or stated behavior fails; private data would be needed to diagnose it. |

For a pass, record only the exact commit/build identifier, architecture, test
bundle identifier, pass/fail signals, and permitted origin. Do not record
accounts, credentials, cookies, tokens, playback history, private screenshots,
or machine settings. A successful pilot check proves only that check; it does
not authorize release, signing, publishing, updater work, data migration, or a
default-launch change.

## Deterministic rollback

Rollback is operationally immediate because the pilot is never a default path:

1. Stop or close the explicitly launched Tauri pilot/test artifact.
2. Launch the unchanged Electron application through its normal `dev` or
   `start` path (or the existing installed Electron application).
3. Do not transfer, delete, or modify either profile's data. The Tauri profile
   remains isolated and Electron continues using its own data and identity.
4. If source rollback is needed, revert only the reviewed pilot commit(s) or
   remove the isolated Tauri seam in a separately reviewed change. Do not
   revert Electron code, packaging, protocol registration, updater, signing,
   or release configuration as part of the pilot rollback.

The rollback completion signal is that Electron still launches through its
unchanged default route and no Tauri command, artifact, protocol, or data path
is selected by that route. A failure to establish that signal blocks the pilot;
it does not permit a cutover workaround.

## Smallest ordered parity backlog

These are proposals, not started implementation work. Each item stays
independently reversible and must keep Electron authoritative.

| Order | Proposed increment | Depends on | Evidence and lane |
| --- | --- | --- | --- |
| 0 | Complete the existing acceptance chain: redacted exact-head HALL-95 observation followed by Clara's HALL-85 decision. | Authorized tester, HALL-94 protocol | **Tessa macOS lane**; no repository behavior change. |
| 1 | Implement HALL-93's deterministic test-bundle inspection, including a controlled negative marker assertion and documented skip/failure behavior. | Order 0 / accepted HALL-85 baseline | Repository-only; no account, signing, or macOS interactive lane. |
| 2 | Add a source-level default-preservation regression check covering Electron launch/release scripts, Electron identity/protocol ownership, and the absence of a default Tauri invocation. | Order 1 | Repository-only; check-only change. |
| 3 | Add one narrow, Tauri-local fresh-profile and navigation-boundary contract test. It must prove no Electron data path is accepted and that only the reviewed YouTube Music/sign-in origins are eligible. | Order 2 and an approved storage/origin contract | Repository-only automated check; its exact reviewed artifact also needs **Tessa macOS lane** launch/origin confirmation. |
| 4 | Consider one user-visible pilot behavior only after Order 3 is accepted, with an exact-head redacted manual check and a scoped rollback test. | Order 3 and a feature-specific contract | **Tessa macOS lane** for launch/behavior observation; repository checks for the boundary. |

Unresolved capability areas in `PARITY_MATRIX.md`—lifecycle, deep links,
settings/data migration, updater, local API, tray view, and full player
control—are not bundled into these increments. They remain Electron-owned until
each receives a separately approved contract.

## Decisions that must be escalated

Do not infer any of the following:

- A new Tauri test package/protocol name or a change to the existing test
  identity; identity and protocol transfer require explicit authorization.
- The exact Tauri profile storage location or any strategy that reads, copies,
  migrates, merges, or deletes Electron data.
- Whether a future pilot may include sign-in, playback, or a new remote origin
  beyond the reviewed policy; account-backed observation requires separately
  authorized Tessa-lane work and the redacted protocol.
- Any signing/notarization, publishing, release channel, updater endpoint/key,
  installation, downgrade, default-launch, or Electron-retirement decision.

Until those decisions exist, ambiguity resolves to preserving Electron as the
default and declining the optional pilot action.
