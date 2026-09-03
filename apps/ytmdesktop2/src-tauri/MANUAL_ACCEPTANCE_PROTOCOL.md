# Redacted manual acceptance protocol

Use this protocol to record non-production, account-backed acceptance evidence
for a reviewed macOS Tauri YouTube Music webview slice. It is deliberately a
recording procedure, not a test plan for the Electron app or a release process.

## Preconditions and limits

- The exact reviewed head must contain the minimal WebKit view being assessed.
  The blank feasibility shell described in `README.md` cannot satisfy the
  navigation, sign-in, or playback observations below.
- The tester must already be separately authorized to use their own YouTube
  account. This document neither grants that authorization nor directs anyone
  to use an account.
- The artifact is unsigned, non-production, and uses its separately reviewed
  test identity. This protocol does not authorize signing, notarization,
  publishing, an updater, a production release, an identity transfer, or a
  default-launch change.
- The tester alone performs sign-in and starts playback. Developers and
  reviewers do not request, handle, or inspect account data.

## Evidence boundary

Record only the fields in the template below. Never collect or commit
credentials, cookies, tokens, account names or identifiers, playback history,
track or playlist names, full URLs, browser/profile data, logs containing such
data, or screenshots containing private account data. A pass/fail result and
the navigation **origin** are sufficient; no account-backed screenshot is
required.

Automated checks and manual evidence are separate. Automated checks establish
the reviewed artifact and preservation boundaries; they cannot prove a
user-directed sign-in or playback result. Conversely, manual evidence does not
replace the automated checks.

## Procedure

1. Before launching the app, record the exact commit, artifact digest or other
   non-secret build identifier, macOS architecture, and bundle identifier.
   Confirm that the artifact is the reviewed unsigned test artifact.
2. Run the reviewed head's existing automated checks and record only their
   outcome. The check set must include:
   - the Tauri artifact-isolation check: no Electron or Chromium payload is
     present in the Tauri artifact; and
   - the Electron-preservation/default-launch check: Electron remains present
     in the repository and the Electron launch path remains the default.

   For the present feasibility seam, `pnpm test` covers the staged test-artifact
   contract and `pnpm --filter ytmdesktop2 tauri:check` covers the isolated
   Tauri configuration. Follow the reviewed slice's documented artifact checks
   when it adds the WebKit view; do not substitute a manual inspection for a
   failing or missing automated check.
3. The authorized tester launches the artifact. Record only whether launch
   succeeded and any non-sensitive failure code or short description.
4. The tester confirms that the WebKit view reaches the allowed YouTube Music
   origin, recorded as `https://music.youtube.com` only. Do not record a path,
   query, redirect URL, or page contents.
5. The tester chooses whether to sign in with their own account. Record only
   `passed`, `failed`, or `not attempted`; never record the account, method,
   prompts, or session details.
6. After a successful sign-in, the tester starts playback themselves. Record
   only whether playback began, without identifying the media. The tester may
   end the session or remove local test data according to their own authorized
   process; this protocol does not prescribe account or data handling.
7. Complete the template and share it only with the exact-head acceptance
   decision. An incomplete template is not a pass and does not authorize a
   release or cutover.

## Evidence template

```text
Manual acceptance evidence (non-production; redacted)

Timestamp (UTC):
Exact commit SHA:
Build/artifact identifier or digest (non-secret):
macOS architecture: arm64 | x86_64
Bundle identifier:
Artifact unsigned: yes | no

Automated artifact-isolation check (no Electron/Chromium): passed | failed | not run
Electron-preservation/default-launch check: passed | failed | not run

Launch result: passed | failed
Navigation origin observed: https://music.youtube.com | failed | not reached
User-directed sign-in result: passed | failed | not attempted
User-directed playback-start result: passed | failed | not attempted

Tester authorization confirmation: I was separately authorized to use my own
YouTube account for this test: yes | no

Non-sensitive notes or failure code (optional):
```

## Acceptance use

Clara may use a completed, internally consistent template to decide acceptance
for the exact recorded head only. A `failed`, `not run`, or required
`not attempted` result leaves acceptance unresolved; it is not evidence of
production readiness. Keep the template redacted when attaching it to an issue
or pull request.

Electron remains the default launch path until a separately authorized cutover.
See `PARITY_MATRIX.md` for the current Electron-retirement boundary and
automated evidence inventory.
