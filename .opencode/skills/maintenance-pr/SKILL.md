---
name: maintenance-pr
description: Run maintenance.sh, automatically apply fixes on a new branch, and create a PR
risk: low
source: repository
---

# Maintenance PR Skill

This skill runs the project's maintenance script, automatically applies required fixes, and creates a pull request with the changes.

## Workflow

### Step 1: Run maintenance.sh
Run the maintenance script to check for issues:
```bash
./maintenance.sh
```

### Step 2: Analyze output
Parse the output for:
- Unused dependencies (shown by cargo-machete)
- Outdated dependencies (shown by cargo outdated)
- Any errors or warnings

### Step 3: Apply fixes
For each issue found:
- **Unused dependencies**: Remove from `Cargo.toml`
- **Outdated dependencies**: Update to latest versions using `cargo update -p <package>` or update version in `Cargo.toml` directly

### Step 4: Create branch
Create a new branch for the changes:
```bash
git checkout -b maintenance/$(date +%Y-%m-%d)
```

### Step 5: Commit and push
Commit the changes and push to remote:
```bash
git add -A && git commit -m "chore: apply maintenance fixes" && git push -u origin HEAD
```

### Step 6: Create PR
Create a pull request using gh CLI:
```bash
gh pr create --title "chore: apply maintenance fixes" --body "Automated maintenance fixes from ./maintenance.sh"
```

## Requirements

- `cargo-machete` (replacement for cargo-udeps)
- `cargo-outdated` (for checking outdated dependencies)
- `gh` CLI (for creating PRs)