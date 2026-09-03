const productionTauriCommand =
  /\b(?:cargo\s+tauri|tauri)\s+(?:build|bundle|package|publish|release|sign|notarize)\b|\bcargo\s+publish\b/i;
const signingOrNotarizationSetting =
  /\b(?:APPLE_ID|APPLE_APP_SPECIFIC_PASSWORD|APPLE_TEAM_ID|CSC_[A-Z_]+|TAURI_SIGNING_PRIVATE_KEY|NOTARIZATION)\b/i;
const protectedDefaultScripts = new Set(["dev", "start", "build"]);
const prohibitedConfigPaths = [
  ["bundle", "createUpdaterArtifacts"],
  ["bundle", "macOS", "signingIdentity"],
  ["bundle", "macOS", "providerShortName"],
  ["bundle", "macOS", "entitlements"],
  ["bundle", "macOS", "notarization"],
  ["plugins", "updater"],
  ["updater"],
];
const prohibitedTauriScriptName = /^tauri:(?:build|bundle|package|publish|release|sign|notarize)$/i;

function valueAtPath(config, path) {
  return path.reduce((value, key) => value?.[key], config);
}

function describePath(path) {
  return path.join(".");
}

function assertAbsent(config, path) {
  if (valueAtPath(config, path) !== undefined) {
    throw new Error(`Tauri configuration must not define ${describePath(path)}`);
  }
}

/**
 * Guards the optional Tauri seam against configuration that would turn it into
 * a distributable application. It only reads parsed manifests and command
 * strings; it never invokes Tauri, Cargo, signing tools, or network services.
 */
export function assertTauriConfigurationIsDevelopmentOnly({ config, appPackage, cargoManifest }) {
  if (config?.bundle?.active !== false) {
    throw new Error("Tauri bundle must remain inactive for the development-only seam");
  }

  for (const path of prohibitedConfigPaths) {
    assertAbsent(config, path);
  }

  if (!/^publish\s*=\s*false\s*$/m.test(cargoManifest)) {
    throw new Error("Tauri Cargo package publishing must remain disabled");
  }

  if (appPackage?.main !== "./out/main/index.js") {
    throw new Error("Tauri must not replace the Electron default entrypoint");
  }

  for (const [name, command] of Object.entries(appPackage?.scripts ?? {})) {
    if (typeof command !== "string") {
      continue;
    }

    if (
      prohibitedTauriScriptName.test(name) ||
      productionTauriCommand.test(command) ||
      signingOrNotarizationSetting.test(command)
    ) {
      throw new Error(`${name} must not build, publish, sign, or notarize Tauri`);
    }

    if (
      protectedDefaultScripts.has(name) &&
      /\b(?:cargo\s+run|cargo\s+tauri|tauri)\b/i.test(command)
    ) {
      throw new Error(`${name} must not make Tauri the default launch path`);
    }
  }
}
