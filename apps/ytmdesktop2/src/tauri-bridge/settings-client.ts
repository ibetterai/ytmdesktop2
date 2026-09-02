import type { PluginBridgeErrorCode } from "./manifest";

export type SettingsKey = "app.zoomFactor";

export type SettingsSnapshotRequest = Readonly<{
	contractVersion: 1;
	keys: readonly SettingsKey[];
}>;

export type SettingsSnapshot = Readonly<{
	contractVersion: 1;
	values: Readonly<{ app?: Readonly<{ zoomFactor: number }> }>;
	missing: readonly SettingsKey[];
}>;

export type SettingsChangedEvent = Readonly<{
	contractVersion: 1;
	key: SettingsKey;
	value: number;
}>;

export type PluginBridgeResult<T> =
	| Readonly<{ ok: true; value: T }>
	| Readonly<{ ok: false; error: Readonly<{ code: PluginBridgeErrorCode }> }>;

/** A named method, deliberately not a generic command transport. */
export type SettingsSnapshotTransport = Readonly<{
	snapshot: (request: SettingsSnapshotRequest) => Promise<PluginBridgeResult<SettingsSnapshot>>;
}>;

export type SettingsChangeEventSource = Readonly<{
	onSettingsChanged: (listener: (event: SettingsChangedEvent) => void) => () => void;
}>;

export type SettingsChangeClient = Readonly<{
	snapshot: () => Promise<PluginBridgeResult<SettingsSnapshot>>;
	onChange: (listener: (event: SettingsChangedEvent) => void) => () => void;
}>;

const SNAPSHOT_REQUEST: SettingsSnapshotRequest = Object.freeze({
	contractVersion: 1,
	keys: Object.freeze(["app.zoomFactor"] as const),
});

export function createSettingsChangeClient(
	transport: SettingsSnapshotTransport,
	events: SettingsChangeEventSource,
): SettingsChangeClient {
	return Object.freeze({
		snapshot: () => transport.snapshot(SNAPSHOT_REQUEST),
		onChange: (listener) => events.onSettingsChanged(listener),
	});
}
