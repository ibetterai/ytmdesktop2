import type { PluginBridgeResult, SettingsChangedEvent } from "./settings-client";

type Dispose = () => void | Promise<void>;

export type TauriPlugin = Readonly<{
	id: string;
	start: () => void | Dispose | Promise<void | Dispose>;
	onSettingsChanged?: (event: SettingsChangedEvent) => void | Promise<void>;
}>;

export type PluginHostState = "idle" | "active" | "disposed" | "failed";

export type PluginHostEvents = Readonly<{
	onChange: (listener: (event: SettingsChangedEvent) => void) => () => void;
}>;

export type DisposablePluginHost = Readonly<{
	start: (plugin: TauriPlugin) => Promise<PluginBridgeResult<undefined>>;
	dispose: () => Promise<PluginBridgeResult<undefined>>;
	state: () => PluginHostState;
}>;

const pluginFailed = (): PluginBridgeResult<never> => ({
	ok: false,
	error: { code: "pluginFailed" },
});

/**
 * Hosts one explicitly supplied plugin. It loads no bundled or third-party
 * plugin itself; selection stays behind the product/security decision gate.
 */
export function createDisposablePluginHost(
	approvedPluginId: string,
	events: PluginHostEvents,
): DisposablePluginHost {
	let state: PluginHostState = "idle";
	let stopListening: (() => void) | undefined;
	let stopPlugin: Dispose | undefined;

	return Object.freeze({
		async start(plugin) {
			if (state !== "idle" || plugin.id !== approvedPluginId) return pluginFailed();

			try {
				const dispose = await plugin.start();
				stopPlugin = typeof dispose === "function" ? dispose : undefined;
				if (plugin.onSettingsChanged) {
					stopListening = events.onChange((event) => {
						void Promise.resolve(plugin.onSettingsChanged?.(event)).catch(() => undefined);
					});
				}
				state = "active";
				return { ok: true, value: undefined };
			} catch {
				state = "failed";
				return pluginFailed();
			}
		},
		async dispose() {
			if (state === "disposed") return { ok: true, value: undefined };

			let failed = false;
			try {
				stopListening?.();
			} catch {
				failed = true;
			}
			try {
				await stopPlugin?.();
			} catch {
				failed = true;
			}
			stopListening = undefined;
			stopPlugin = undefined;

			if (failed) {
				state = "failed";
				return pluginFailed();
			}

			state = "disposed";
			return { ok: true, value: undefined };
		},
		state: () => state,
	});
}
