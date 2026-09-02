#!/usr/bin/env bash
set -euo pipefail

status=0

if [ "${PUBLIC_AUDIT_SKIP_GITLEAKS:-0}" = "1" ]; then
  echo "public-audit: gitleaks skipped here; GitHub Actions ran the dedicated scanner step" >&2
elif command -v gitleaks >/dev/null 2>&1; then
  gitleaks detect --source . --redact --no-banner --exit-code 99 || status=$?
  if [ "$status" -ne 0 ]; then
    echo "public-audit: gitleaks found possible secrets" >&2
  fi
else
  echo "public-audit: gitleaks not installed; GitHub Actions runs the secret scan separately" >&2
fi

tracked_artifacts=$(git ls-files | grep -E '(^|/)(tmp|logs?)(/|$)|[.](dmg|pkg|log)$' || true)
if [ -n "$tracked_artifacts" ]; then
  echo "public-audit: generated artifacts/logs are tracked:" >&2
  echo "$tracked_artifacts" >&2
  status=1
fi

pattern='(/Users/[^[:space:]/]+|/home/[^[:space:]/]+|C:[\\/]+Users[\\/]+|(^|[^0-9])(10[.][0-9]{1,3}[.][0-9]{1,3}[.][0-9]{1,3}|169[.]254[.][0-9]{1,3}[.][0-9]{1,3}|192[.]168[.][0-9]{1,3}[.][0-9]{1,3}|172[.](1[6-9]|2[0-9]|3[01])[.][0-9]{1,3}[.][0-9]{1,3})([^0-9]|$))'
if command -v rg >/dev/null 2>&1; then
  if rg -n --hidden --glob '!.git/**' --glob '!scripts/public-audit.sh' --glob '!apps/ytmdesktop2/src/shared/chromecast/endpoint.test.ts' --glob '!**/*.png' --glob '!**/*.jpg' --glob '!**/*.jpeg' --glob '!**/*.gif' --glob '!**/*.icns' --glob '!**/*.ico' --glob '!**/*.svg' "$pattern" .; then
    echo "public-audit: possible local path or private network address found" >&2
    status=1
  fi
else
  echo "public-audit: ripgrep not installed; skipping generic private-path/IP scan" >&2
fi

exit "$status"
