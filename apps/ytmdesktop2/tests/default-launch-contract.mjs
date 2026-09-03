const defaultScriptNames = ["dev", "start", "build"];
const electronEntrypoint = "./out/main/index.js";
const electronTooling = /\belectron-(?:vite|builder)\b/i;
const tauriInvocation = /\btauri(?::|\b)/i;

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
