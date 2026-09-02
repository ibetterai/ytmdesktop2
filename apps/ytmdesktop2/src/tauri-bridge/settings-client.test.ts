import { describe, expect, it, vi } from "vitest";
import { createSettingsChangeClient } from "./settings-client";

describe("Tauri settings change client", () => {
	it("uses only the versioned snapshot request and relays typed changes", async () => {
		const snapshot = vi.fn().mockResolvedValue({
			ok: true as const,
			value: {
				contractVersion: 1 as const,
				values: { app: { zoomFactor: 1 } },
				missing: [],
			},
		});
		let listener: ((event: { contractVersion: 1; key: "app.zoomFactor"; value: number }) => void) | undefined;
		const client = createSettingsChangeClient(
			{ snapshot },
			{
				onSettingsChanged(next) {
					listener = next;
					return () => {
						listener = undefined;
					};
				},
			},
		);

		await expect(client.snapshot()).resolves.toEqual({
			ok: true,
			value: {
				contractVersion: 1,
				values: { app: { zoomFactor: 1 } },
				missing: [],
			},
		});
		expect(snapshot).toHaveBeenCalledWith({
			contractVersion: 1,
			keys: ["app.zoomFactor"],
		});

		const onChange = vi.fn();
		const dispose = client.onChange(onChange);
		listener?.({ contractVersion: 1, key: "app.zoomFactor", value: 1.25 });
		expect(onChange).toHaveBeenCalledWith({ contractVersion: 1, key: "app.zoomFactor", value: 1.25 });
		dispose();
		expect(listener).toBeUndefined();
	});
});
