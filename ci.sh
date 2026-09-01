#!/usr/bin/env bash
# Per-repo CI shim. The fleet runner invokes this file from the PR branch and
# supplies a pinned copy of the shared CI library as its first argument.
#
# This repository is a pnpm workspace. The shared library has no pnpm stack and
# otherwise treats package.json as npm, which installs only the root package and
# leaves workspace tooling unavailable. Preserve its path gate, gitleaks, and
# workflow-shell checks; replace only its npm stack sequence with the equivalent
# pinned-pnpm commands.
set -euo pipefail

LIB="${1:?usage: ci.sh <path-to-ci-lib.sh>}"
# shellcheck source=/dev/null
source "$LIB"

pnpm_command() {
  local package_manager
  package_manager=$(node -p "require('./package.json').packageManager || ''")
  case "$package_manager" in
    pnpm@*) ;;
    *)
      echo "ci:pnpm packageManager pin missing from package.json" >&2
      return 1
      ;;
  esac

  if command -v pnpm >/dev/null 2>&1 && [ "$(pnpm --version)" = "${package_manager#pnpm@}" ]; then
    pnpm "$@"
  else
    npx --yes "$package_manager" "$@"
  fi
}

ci_run_npm() {
  ci_run_step "pnpm install" pnpm_command install --frozen-lockfile || return 1
  ci_run_step "pnpm lint" pnpm_command lint || return 1
  ci_run_step "pnpm typecheck" pnpm_command typecheck || return 1

  if [ -f package.json ] && grep -q '"@playwright/test"' package.json; then
    echo "ci:pnpm Playwright test suite — running without --coverage (it rejects the flag)."
    ci_run_step "pnpm test" pnpm_command test || return 1
  else
    ci_run_step "pnpm test" pnpm_command test -- --coverage || return 1
  fi

  ci_start "pnpm build"
  if [ "${CI_SKIP_BUILD:-}" = "1" ]; then
    echo "ci:pnpm skip build (CI_SKIP_BUILD=1 — run-build: false equivalent)"
    ci_end "pnpm build"
  elif [ "${CI_EVENT:-}" = "push" ] && [ "$CI_OPENNEXT" = "true" ]; then
    echo "ci:pnpm skip build (OpenNext repo on push — the deploy workflow builds the worker)"
    ci_end "pnpm build"
  else
    local heap_mb
    heap_mb=$(ci_npm_build_heap_mb)
    echo "ci:pnpm build heap cap ${heap_mb}MB (NODE_OPTIONS --max-old-space-size, sized from MemAvailable)"
    if NODE_OPTIONS=--max-old-space-size=$heap_mb pnpm_command build; then
      ci_end "pnpm build"
    else
      ci_end "pnpm build" "FAIL"
      return 1
    fi
  fi
}

# ci_main accepts npm/bun/python. Passing npm selects the overridden function
# above; ci_main still performs shared detection for CI_OPENNEXT and runs all
# non-stack checks.
ci_main npm "${CI_BASE_SHA:?CI_BASE_SHA is required}" "${CI_HEAD_SHA:?CI_HEAD_SHA is required}"
