import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

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
  const [config, appPackage, readme] = await Promise.all([
    readJson(join(tauriDirectory, "tauri.conf.json")),
    readJson(join(appDirectory, "package.json")),
    readFile(join(tauriDirectory, "README.md"), "utf8"),
  ]);

  assert.deepEqual(config.build, { frontendDist: "ui" });
  assert.deepEqual(config.bundle, { active: false, icon: ["icons/icon.png"] });
  assert.equal(
    appPackage.scripts["tauri:source-check"],
    "node --test src-tauri/tests/source-config.test.mjs",
  );
  assert.match(readme, /`tauri:source-check` reads only committed source files/);
  assert.match(
    readme,
    /does not build, sign, launch, or inspect a generated\s+application\s+bundle/,
  );
});
