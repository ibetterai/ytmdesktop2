import releaseArtifactManifest from "../src-tauri/release-artifact-manifest.json";

export type ReleaseArtifact = Readonly<{
  platform: "macos" | "windows" | "linux";
  arch: string;
  version: string;
  channel: "test";
  identifier: string;
  updateEndpoint?: never;
}>;

export type ReleaseValidationError = Readonly<{
  code: "identityMismatch" | "unsupportedTarget" | "unsignedArtifact";
}>;

type StagedTauriTestArtifact = Readonly<{
  artifact: ReleaseArtifact;
  isSigned: boolean;
}>;

export const TAURI_TEST_RELEASE_ARTIFACT_PATHS = Object.freeze({
  manifest: "src-tauri/release-artifact-manifest.json",
  outputDirectory: "src-tauri/target/test-artifacts",
});

/**
 * The only release-shaped metadata available to the feasibility shell.
 * It is a test-channel manifest and deliberately cannot carry an update endpoint.
 */
export const TAURI_TEST_RELEASE_ARTIFACT = Object.freeze(
  releaseArtifactManifest as ReleaseArtifact,
);

/**
 * Validates the fixed test/staging artifact contract without reading paths, publishing,
 * or exposing signing details. The unsigned result makes this artifact ineligible for release.
 */
export function verifyStagedTauriTestArtifact(
  stagedArtifact: StagedTauriTestArtifact,
): readonly ReleaseValidationError[] {
  const errors: ReleaseValidationError[] = [];
  const { artifact } = stagedArtifact;

  if (artifact.identifier !== TAURI_TEST_RELEASE_ARTIFACT.identifier) {
    errors.push({ code: "identityMismatch" });
  }

  if (
    artifact.platform !== TAURI_TEST_RELEASE_ARTIFACT.platform ||
    artifact.arch !== TAURI_TEST_RELEASE_ARTIFACT.arch
  ) {
    errors.push({ code: "unsupportedTarget" });
  }

  if (!stagedArtifact.isSigned) {
    errors.push({ code: "unsignedArtifact" });
  }

  return errors;
}
