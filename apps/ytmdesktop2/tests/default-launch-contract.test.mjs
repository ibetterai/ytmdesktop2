import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  assertElectronDefaultLaunch,
  assertTauriDeveloperLaunchIsOptIn,
} from "./default-launch-contract.mjs";

const appDirectory = dirname(dirname(fileURLToPath(import.meta.url)));
const repositoryDirectory = dirname(dirname(appDirectory));

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

function createValidManifests() {
  return {
    workspacePackage: {
      scripts: {
        dev: "pnpm --filter ytmdesktop2 dev",
        start: "pnpm --filter ytmdesktop2 start",
        build: "pnpm --filter ytmdesktop2 build",
        "build:mac": "pnpm --filter ytmdesktop2 build:mac",
      },
    },
    desktopPackage: {
      main: "./out/main/index.js",
      scripts: {
        dev: "electron-vite dev --watch",
        start: "electron-vite preview",
        build: "pnpm typecheck && electron-vite build",
        "build:mac": "pnpm build && electron-builder --mac",
      },
    },
  };
}

function createIsolatedTauriLaunchConfiguration() {
  return {
    desktopPackage: {
      scripts: {
        "tauri:dev": "cargo run --manifest-path src-tauri/Cargo.toml",
      },
    },
    electronBuilderConfig: `appId: net.venipa.ytmdesktop
protocols:
  - name: YouTube Music for Desktop
    schemes:
      - ytmd
`,
    tauriConfig: {
      identifier: "com.ibetterai.ytmdesktop2.tauri-spike",
    },
  };
}

test("keeps the committed Electron entrypoint and ordinary launch scripts as default", async () => {
  const [workspacePackage, desktopPackage] = await Promise.all([
    readJson(join(repositoryDirectory, "package.json")),
    readJson(join(appDirectory, "package.json")),
  ]);

  assert.doesNotThrow(() => assertElectronDefaultLaunch({ workspacePackage, desktopPackage }));
});

test("keeps the Tauri developer launcher opt-in and preserves Electron identity ownership", async () => {
  const [desktopPackage, tauriConfig, electronBuilderConfig] = await Promise.all([
    readJson(join(appDirectory, "package.json")),
    readJson(join(appDirectory, "src-tauri", "tauri.conf.json")),
    readFile(join(appDirectory, "electron-builder.yml"), "utf8"),
  ]);

  assert.doesNotThrow(() =>
    assertTauriDeveloperLaunchIsOptIn({ desktopPackage, electronBuilderConfig, tauriConfig }),
  );
});

test("rejects an unnamed Tauri launcher and identity or protocol ownership changes", () => {
  const configuration = createIsolatedTauriLaunchConfiguration();

  assert.throws(
    () =>
      assertTauriDeveloperLaunchIsOptIn({
        ...configuration,
        desktopPackage: {
          scripts: { ...configuration.desktopPackage.scripts, "dev:tauri": "cargo run" },
        },
      }),
    /Tauri developer launchers must use an explicit tauri: script name/,
  );
  assert.throws(
    () =>
      assertTauriDeveloperLaunchIsOptIn({
        ...configuration,
        electronBuilderConfig: configuration.electronBuilderConfig.replace(
          "net.venipa.ytmdesktop",
          "com.ibetterai.ytmdesktop2.tauri-spike",
        ),
      }),
    /Electron appId must remain net\.venipa\.ytmdesktop/,
  );
  assert.throws(
    () =>
      assertTauriDeveloperLaunchIsOptIn({
        ...configuration,
        electronBuilderConfig: configuration.electronBuilderConfig.replace("- ytmd", "- tauri"),
      }),
    /Electron must remain the owner of the ytmd protocol/,
  );
  assert.throws(
    () =>
      assertTauriDeveloperLaunchIsOptIn({
        ...configuration,
        tauriConfig: { identifier: "net.venipa.ytmdesktop" },
      }),
    /Tauri identifier must remain distinct from Electron appId/,
  );
  assert.throws(
    () =>
      assertTauriDeveloperLaunchIsOptIn({
        ...configuration,
        tauriConfig: {
          identifier: "com.ibetterai.ytmdesktop2.tauri-spike",
          plugins: { "deep-link": { schemes: ["ytmd"] } },
        },
      }),
    /Tauri must not claim Electron's protocol through configuration/,
  );
});

test("permits an absent optional-Tauri command set", () => {
  const result = assertElectronDefaultLaunch(createValidManifests());

  assert.deepEqual(result.optionalTauriCommands, []);
});

test("rejects Tauri redirects for the entrypoint and default launch commands", () => {
  const { workspacePackage, desktopPackage } = createValidManifests();

  assert.throws(
    () =>
      assertElectronDefaultLaunch({
        workspacePackage,
        desktopPackage: { ...desktopPackage, main: "./src-tauri/main.rs" },
      }),
    /desktop main must remain the Electron output entrypoint/,
  );
  assert.throws(
    () =>
      assertElectronDefaultLaunch({
        workspacePackage: {
          ...workspacePackage,
          scripts: { ...workspacePackage.scripts, dev: "pnpm --filter ytmdesktop2-tauri dev" },
        },
        desktopPackage,
      }),
    /workspace dev must delegate to the Electron desktop package/,
  );
  assert.throws(
    () =>
      assertElectronDefaultLaunch({
        workspacePackage: {
          ...workspacePackage,
          scripts: {
            ...workspacePackage.scripts,
            "build:mac": "pnpm --filter ytmdesktop2-tauri build",
          },
        },
        desktopPackage,
      }),
    /workspace build:mac must delegate to the Electron desktop package/,
  );
  assert.throws(
    () =>
      assertElectronDefaultLaunch({
        workspacePackage,
        desktopPackage: {
          ...desktopPackage,
          scripts: { ...desktopPackage.scripts, dev: "pnpm tauri:dev" },
        },
      }),
    /desktop dev must not invoke Tauri/,
  );
});
