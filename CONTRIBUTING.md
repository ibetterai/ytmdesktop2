# Contributing

## Pull requests

Open changes against `main`. Every pull request must receive one fresh approval and pass
three public checks: **Unit tests**, **Tauri feasibility**, and **public-audit**. New commits
dismiss earlier approvals, and review conversations must be resolved.

Non-draft pull requests are automatically armed for squash auto-merge. GitHub merges the
change and deletes its branch only after every review and check requirement is satisfied.
No separate merge click is needed after approval.

## Upstream synchronization

This fork's `main` tracks `master` in `Venipa/ytmdesktop2`. Sync the differently named
branches explicitly:

```bash
git fetch upstream
git switch main
git merge upstream/master
```

## Local setup

Follow the setup instructions in [README.md](README.md). The workspace uses the package
manager version pinned in the root `package.json`.
