import { describe, expect, it } from "vitest";
import {
  TAURI_TEST_RELEASE_ARTIFACT,
  TAURI_TEST_RELEASE_ARTIFACT_PATHS,
  verifyStagedTauriTestArtifact,
} from "./tauri-release-artifact";

describe("Tauri test release artifact", () => {
  it("publishes a fixed, endpoint-free manifest for the unsigned macOS test artifact", () => {
    expect(TAURI_TEST_RELEASE_ARTIFACT).toEqual({
      platform: "macos",
      arch: "aarch64",
      version: "0.0.0",
      channel: "test",
      identifier: "com.ibetterai.ytmdesktop2.tauri-spike",
    });
    expect(TAURI_TEST_RELEASE_ARTIFACT).not.toHaveProperty("updateEndpoint");
    expect(Object.isFrozen(TAURI_TEST_RELEASE_ARTIFACT)).toBe(true);
    expect(TAURI_TEST_RELEASE_ARTIFACT_PATHS).toEqual({
      inputManifest: "src-tauri/target/test-artifacts/input/release-artifact-manifest.json",
      outputDirectory: "src-tauri/target/test-artifacts/output",
    });
  });

  it("reports only stable validation codes for staging artifacts", () => {
    expect(
      verifyStagedTauriTestArtifact({
        artifact: TAURI_TEST_RELEASE_ARTIFACT,
        isSigned: false,
      }),
    ).toEqual([{ code: "unsignedArtifact" }]);

    expect(
      verifyStagedTauriTestArtifact({
        artifact: { ...TAURI_TEST_RELEASE_ARTIFACT, identifier: "net.venipa.ytmdesktop" },
        isSigned: false,
      }),
    ).toEqual([{ code: "identityMismatch" }, { code: "unsignedArtifact" }]);

    expect(
      verifyStagedTauriTestArtifact({
        artifact: { ...TAURI_TEST_RELEASE_ARTIFACT, platform: "windows" },
        isSigned: true,
      }),
    ).toEqual([{ code: "unsupportedTarget" }]);
  });
});
