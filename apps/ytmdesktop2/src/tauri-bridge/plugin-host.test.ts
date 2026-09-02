import { describe, expect, it, vi } from "vitest";
import { createDisposablePluginHost } from "./plugin-host";

describe("disposable Tauri plugin host", () => {
	it("unsubscribes from settings before disposing its only plugin", async () => {
		const cleanup = vi.fn();
		const unsubscribe = vi.fn();
		const host = createDisposablePluginHost("future-named-plugin", {
			onChange: () => unsubscribe,
		});

		await expect(
			host.start({
				id: "future-named-plugin",
				start: () => cleanup,
				onSettingsChanged: () => undefined,
			}),
		).resolves.toEqual({ ok: true });
		await expect(host.dispose()).resolves.toEqual({ ok: true });

		expect(unsubscribe.mock.invocationCallOrder[0]).toBeLessThan(cleanup.mock.invocationCallOrder[0]);
		expect(host.state()).toBe("disposed");
	});

	it("contains startup and cleanup failures behind pluginFailed", async () => {
		const failedStart = createDisposablePluginHost("future-named-plugin", { onChange: () => () => undefined });
		await expect(
			failedStart.start({ id: "future-named-plugin", start: () => Promise.reject(new Error("secret")) }),
		).resolves.toEqual({ ok: false, error: { code: "pluginFailed" } });

		const failedCleanup = createDisposablePluginHost("future-named-plugin", { onChange: () => () => undefined });
		await failedCleanup.start({ id: "future-named-plugin", start: () => () => Promise.reject(new Error("secret")) });
		await expect(failedCleanup.dispose()).resolves.toEqual({ ok: false, error: { code: "pluginFailed" } });
	});

	it("will not start an unapproved plugin id", async () => {
		const host = createDisposablePluginHost("future-named-plugin", { onChange: () => () => undefined });

		await expect(host.start({ id: "unapproved", start: () => undefined })).resolves.toEqual({
			ok: false,
			error: { code: "pluginFailed" },
		});
	});

	it("still disposes the plugin when settings unsubscription fails", async () => {
		const cleanup = vi.fn();
		const host = createDisposablePluginHost("future-named-plugin", {
			onChange: () => () => {
				throw new Error("listener cleanup failed");
			},
		});
		await host.start({
			id: "future-named-plugin",
			start: () => cleanup,
			onSettingsChanged: () => undefined,
		});

		await expect(host.dispose()).resolves.toEqual({ ok: false, error: { code: "pluginFailed" } });
		expect(cleanup).toHaveBeenCalledOnce();
	});
});
