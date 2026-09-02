import bridgeManifest from "../../src-tauri/bridge-manifest.json";

export const TAURI_PLUGIN_BRIDGE_CAPABILITY = bridgeManifest.callers[0].capability as "allow-tauri-plugin-bridge";
export const TAURI_PLUGIN_BRIDGE_ORIGIN = bridgeManifest.callers[0].origin as "tauri://localhost";

export type PluginBridgeErrorCode =
	| "unsupportedCommand"
	| "unauthorizedCaller"
	| "invalidRequest"
	| "pluginFailed";

type ManifestCaller = Readonly<{
	webview: "main";
	origin: typeof TAURI_PLUGIN_BRIDGE_ORIGIN;
	capability: typeof TAURI_PLUGIN_BRIDGE_CAPABILITY;
}>;

type ManifestMethod = Readonly<{
	name: "settings.snapshot";
	request: "SettingsSnapshotRequest";
	response: "SettingsSnapshot";
	errors: readonly PluginBridgeErrorCode[];
}>;

type ManifestEvent = Readonly<{
	name: "settings.changed";
	payload: "SettingsChangedEvent";
	callers: readonly ["main"];
}>;

export type TauriPluginBridgeManifest = Readonly<{
	contractVersion: 1;
	callers: readonly ManifestCaller[];
	methods: readonly ManifestMethod[];
	events: readonly ManifestEvent[];
}>;

function deepFreeze<T>(value: T): Readonly<T> {
	if (value && typeof value === "object") {
		for (const child of Object.values(value)) deepFreeze(child);
		Object.freeze(value);
	}
	return value;
}

/**
 * The inactive feasibility bridge's full public surface. Adding a webview,
 * origin, method, or event requires an explicit manifest review.
 */
export const TAURI_PLUGIN_BRIDGE_MANIFEST = deepFreeze(bridgeManifest) as TauriPluginBridgeManifest;
