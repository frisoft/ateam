---
name: release
description: Creates a git tag to trigger the release workflow. The release workflow builds binaries, creates a GitHub Release, and publishes to crates.io.
---

# Release Skill

This skill creates a git tag to trigger the release workflow.

## Prerequisites

Before using this skill, you must:
1. Bump the version in `Cargo.toml`
2. Update `CHANGELOG.md` with the new version and release notes
3. Commit and push those changes to the main branch

## When to use

Use this skill when you want to release a new version to crates.io.

## Implementation

When this skill is invoked, perform the following steps:

### Step 1: Get the version
Run `grep -m1 '^version' Cargo.toml` to get the current version from Cargo.toml.

### Step 2: Create the version tag
Create a git tag with the version from Step 4:
- Format: `v<version>` (e.g., `v1.0.14`)
- Run `git tag v<version>`

### Step 3: Push the tag to remote
Push the tag to the remote repository:
- Run `git push origin v<version>`
- This triggers the release workflow in `.github/workflows/release.yml`

## Output

- Print the version found in Cargo.toml
- Print the tag created
- Print confirmation of the push
- Explain that the release workflow will automatically build, create GitHub Release, and publish to crates.io