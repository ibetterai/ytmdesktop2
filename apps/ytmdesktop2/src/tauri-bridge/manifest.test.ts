import { describe, expect, it } from "vitest";
import {
	TAURI_PLUGIN_BRIDGE_CAPABILITY,
	TAURI_PLUGIN_BRIDGE_MANIFEST,
	TAURI_PLUGIN_BRIDGE_ORIGIN,
} from "./manifest";

describe("Tauri plugin bridge manifest", () => {
	it("declares only the main Tauri origin, one command, and one event", () => {
		expect(TAURI_PLUGIN_BRIDGE_MANIFEST).toEqual({
			contractVersion: 1,
			callers: [
				{
					webview: "main",
					origin: TAURI_PLUGIN_BRIDGE_ORIGIN,
					capability: TAURI_PLUGIN_BRIDGE_CAPABILITY,
				},
			],
			methods: [
				{
					name: "settings.snapshot",
					request: "SettingsSnapshotRequest",
					response: "SettingsSnapshot",
					errors: ["unsupportedCommand", "unauthorizedCaller", "invalidRequest", "pluginFailed"],
				},
			],
			events: [
				{
					name: "settings.changed",
					payload: "SettingsChangedEvent",
					callers: ["main"],
				},
			],
		});
	});

	it("is immutable through every manifest level", () => {
		expect(Object.isFrozen(TAURI_PLUGIN_BRIDGE_MANIFEST)).toBe(true);
		expect(Object.isFrozen(TAURI_PLUGIN_BRIDGE_MANIFEST.callers)).toBe(true);
		expect(Object.isFrozen(TAURI_PLUGIN_BRIDGE_MANIFEST.callers[0])).toBe(true);
		expect(Object.isFrozen(TAURI_PLUGIN_BRIDGE_MANIFEST.methods[0].errors)).toBe(true);
	});
});
