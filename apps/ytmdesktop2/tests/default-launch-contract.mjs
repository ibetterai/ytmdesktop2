const defaultScriptNames = ["dev", "start", "build"];
const electronEntrypoint = "./out/main/index.js";
const electronTooling = /\belectron-(?:vite|builder)\b/i;
const tauriInvocation = /\btauri(?::|\b)/i;
const tauriDeveloperLaunch = /\b(?:cargo\s+run|tauri\s+dev)\b/i;
const electronAppId = "net.venipa.ytmdesktop";
const tauriSpikeIdentifier = "com.ibetterai.ytmdesktop2.tauri-spike";

function requireScript(scripts, scriptName, packageName) {
  const command = scripts?.[scriptName];
  if (typeof command !== "string" || command.length === 0) {
    throw new Error(`${packageName} ${scriptName} must remain defined`);
  }

  return command;
}

function assertNoTauriInvocation(command, label) {
  if (tauriInvocation.test(command)) {
    throw new Error(`${label} must not invoke Tauri`);
  }
}

function assertWorkspaceDelegation(workspaceScripts, scriptName) {
  const command = requireScript(workspaceScripts, scriptName, "workspace");
  const expectedCommand = `pnpm --filter ytmdesktop2 ${scriptName}`;

  if (command !== expectedCommand) {
    throw new Error(`workspace ${scriptName} must delegate to the Electron desktop package`);
  }

  assertNoTauriInvocation(command, `workspace ${scriptName}`);
}

function assertElectronTooling(desktopScripts, scriptName, visitedScripts = new Set()) {
  const command = requireScript(desktopScripts, scriptName, "desktop");

  assertNoTauriInvocation(command, `desktop ${scriptName}`);
  if (electronTooling.test(command)) {
    return;
  }

  const alias = command.match(/\bpnpm(?: run)? (dev|start|build)\b/)?.[1];
  if (alias && !visitedScripts.has(alias)) {
    visitedScripts.add(scriptName);
    assertElectronTooling(desktopScripts, alias, visitedScripts);
    return;
  }

  throw new Error(`${scriptName} must invoke Electron tooling`);
}

/**
 * Verifies the committed package-manifest launch boundary without launching,
 * building, or packaging either desktop implementation.
 */
export function assertElectronDefaultLaunch({ workspacePackage, desktopPackage }) {
  if (desktopPackage?.main !== electronEntrypoint) {
    throw new Error("desktop main must remain the Electron output entrypoint");
  }

  const workspaceScripts = workspacePackage?.scripts;
  const desktopScripts = desktopPackage?.scripts;

  for (const scriptName of defaultScriptNames) {
    assertWorkspaceDelegation(workspaceScripts, scriptName);
    assertElectronTooling(desktopScripts, scriptName);
  }

  for (const scriptName of Object.keys(workspaceScripts ?? {}).filter(
    (name) => name.startsWith("release") || name.startsWith("build:"),
  )) {
    assertWorkspaceDelegation(workspaceScripts, scriptName);
  }

  for (const scriptName of Object.keys(desktopScripts ?? {}).filter(
    (name) => name.startsWith("release") || name.startsWith("build:"),
  )) {
    assertElectronTooling(desktopScripts, scriptName);
  }

  return {
    optionalTauriCommands: Object.keys(desktopScripts ?? {}).filter((name) =>
      name.startsWith("tauri:"),
    ),
  };
}

/**
 * Verifies the explicit Tauri developer opt-in and the Electron identity/
 * deep-link configuration without executing either desktop implementation.
 */
export function assertTauriDeveloperLaunchIsOptIn({
  desktopPackage,
  electronBuilderConfig,
  tauriConfig,
}) {
  const scripts = desktopPackage?.scripts ?? {};
  const developerLaunchers = Object.entries(scripts).filter(([, command]) =>
    tauriDeveloperLaunch.test(command),
  );

  if (developerLaunchers.length === 0) {
    throw new Error("an explicit tauri:dev developer launcher must remain defined");
  }

  for (const [name] of developerLaunchers) {
    if (!name.startsWith("tauri:")) {
      throw new Error("Tauri developer launchers must use an explicit tauri: script name");
    }
  }

  if (!tauriDeveloperLaunch.test(scripts["tauri:dev"] ?? "")) {
    throw new Error("tauri:dev must remain the explicit Tauri developer launcher");
  }

  if (!/^appId:\s*net\.venipa\.ytmdesktop\s*$/m.test(electronBuilderConfig)) {
    throw new Error(`Electron appId must remain ${electronAppId}`);
  }

  if (
    !/protocols:\s*\n\s*-\s+name:[^\n]*\n\s+schemes:\s*\n\s+-\s+ytmd\s*$/m.test(
      electronBuilderConfig,
    )
  ) {
    throw new Error("Electron must remain the owner of the ytmd protocol");
  }

  if (tauriConfig?.identifier !== tauriSpikeIdentifier) {
    throw new Error("Tauri identifier must remain distinct from Electron appId");
  }

  if (
    tauriConfig?.plugins?.["deep-link"] !== undefined ||
    tauriConfig?.app?.deepLink !== undefined
  ) {
    throw new Error("Tauri must not claim Electron's protocol through configuration");
  }

  return { developerLaunchers: developerLaunchers.map(([name]) => name) };
}
