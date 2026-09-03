import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { assertTauriConfigurationIsDevelopmentOnly } from "./development-only-config.mjs";

const tauriDirectory = dirname(dirname(fileURLToPath(import.meta.url)));
const appDirectory = dirname(tauriDirectory);
const expectedIdentifier = "com.ibetterai.ytmdesktop2.tauri-spike";

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

test("pins the isolated Tauri application identity", async () => {
  const config = await readJson(join(tauriDirectory, "tauri.conf.json"));

  assert.equal(config.productName, "YTMDesktop2 Tauri Feasibility");
  assert.equal(config.version, "0.0.0");
  assert.equal(config.identifier, expectedIdentifier);
});

test("pins macOS arm64 intent to the distinct test artifact identity", async () => {
  const artifact = await readJson(join(tauriDirectory, "release-artifact-manifest.json"));

  assert.deepEqual(artifact, {
    platform: "macos",
    arch: "aarch64",
    version: "0.0.0",
    channel: "test",
    identifier: expectedIdentifier,
  });
});

test("keeps development local-only and documents the standalone source check", async () => {
  const [config, appPackage, cargoManifest, readme] = await Promise.all([
    readJson(join(tauriDirectory, "tauri.conf.json")),
    readJson(join(appDirectory, "package.json")),
    readFile(join(tauriDirectory, "Cargo.toml"), "utf8"),
    readFile(join(tauriDirectory, "README.md"), "utf8"),
  ]);

  assert.deepEqual(config.build, { frontendDist: "ui" });
  assert.deepEqual(config.bundle, { active: false, icon: ["icons/icon.png"] });
  assert.doesNotThrow(() =>
    assertTauriConfigurationIsDevelopmentOnly({ config, appPackage, cargoManifest }),
  );
  assert.equal(
    appPackage.scripts["tauri:source-check"],
    "node --test src-tauri/tests/source-config.test.mjs",
  );
  assert.match(readme, /`tauri:source-check` reads only committed source files/);
  assert.match(
    readme,
    /does not build, sign, launch, or inspect a generated\s+application\s+bundle/,
  );
  assert.match(readme, /development-only guard rejects Tauri production configuration/);
});

test("rejects distribution, signing, updater, publishing, and default-launch configuration", () => {
  const developmentConfig = {
    bundle: { active: false },
  };
  const developmentPackage = {
    main: "./out/main/index.js",
    scripts: {
      "tauri:source-check": "node --test src-tauri/tests/source-config.test.mjs",
      "tauri:check": "pnpm tauri:source-check && cargo test --locked",
      "tauri:dev": "cargo run --manifest-path src-tauri/Cargo.toml",
    },
  };
  const developmentCargoManifest = "[package]\npublish = false\n";

  const prohibitedConfigurations = [
    {
      config: { bundle: { active: true } },
      error: /bundle must remain inactive/,
    },
    {
      config: { bundle: { active: false, macOS: { signingIdentity: "Developer ID" } } },
      error: /signingIdentity/,
    },
    {
      config: { bundle: { active: false, macOS: { notarization: true } } },
      error: /notarization/,
    },
    {
      config: { bundle: { active: false, createUpdaterArtifacts: true } },
      error: /createUpdaterArtifacts/,
    },
    {
      config: { bundle: { active: false }, plugins: { updater: { endpoints: ["https://x"] } } },
      error: /plugins\.updater/,
    },
    {
      appPackage: { main: "./out/main/index.js", scripts: { "tauri:build": "echo pending" } },
      error: /tauri:build must not build/,
    },
    {
      appPackage: {
        main: "./out/main/index.js",
        scripts: { dev: "cargo run --manifest-path src-tauri/Cargo.toml" },
      },
      error: /dev must not make Tauri the default launch path/,
    },
    {
      appPackage: { main: "./src-tauri/main.rs", scripts: {} },
      error: /must not replace the Electron default entrypoint/,
    },
    {
      cargoManifest: "[package]\npublish = true\n",
      error: /Cargo package publishing must remain disabled/,
    },
  ];

  for (const prohibited of prohibitedConfigurations) {
    assert.throws(
      () =>
        assertTauriConfigurationIsDevelopmentOnly({
          config: prohibited.config ?? developmentConfig,
          appPackage: prohibited.appPackage ?? developmentPackage,
          cargoManifest: prohibited.cargoManifest ?? developmentCargoManifest,
        }),
      prohibited.error,
    );
  }
});
