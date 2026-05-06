---
name: pusher
description: Stages, commits, and pushes changes to git. Runs cargo fmt before staging to ensure code is properly formatted. Creates a branch if on main and creates a PR if none exists.
---

# Pusher Skill

This skill stages, commits, and pushes changes to git.

## Essential Rule

**NEVER commit or push directly to the main branch.** Always create a feature branch first.

## Description

Format the source code with cargo fmt, then stage changes with git add, create a commit with git commit, and push to the remote with git push. Ensures changes are never pushed directly to main branch - creates a branch if on main, and creates a PR if none exists for the branch.

## When to use

Use this skill when you want to commit and push changes to the repository.


## Implementation

When this skill is invoked, perform the following steps:

### Step 1: Check current branch
Run `git branch --show-current` to determine the current branch.

### Step 2: Handle main branch (REQUIRED)
If on main branch:
- Changes to main brnach are **NOT allowed**
- Create a feature brach but do not ask the user for a new branch name
- Generate a branch name automatically based on the changes:
  - Check git status or diff to understand what changed
  - Use format: `<type>/<short-description>` where type is: feature/, fix/, refactor/, docs/, test/
  - Example: if adding tests, use `test/add-phase5-integration-tests`
- Create and checkout the new branch with `git checkout -b <branch-name>`
- Set upstream with `git push -u origin <branch-name>`

### Step 3: Format the source code
Run `cargo fmt -- --check` to check if the code is formatted correctly.

If the code is not formatted correctly, run `cargo fmt` to format it automatically.

### Step 4: Stage the changes
Run `git add -A` to stage all changes.

### Step 5: Check the status
Run `git status` to see what will be committed.

### Step 6: Create a commit
Run `git commit` with an auto-generated commit message:
- Derive the message from the changes: summarize what files were changed and what was done
- Use conventional commit format: `<type>: <description>` where type is: feat, fix, refactor, docs, test, chore
- Examples:
  - "test: add phase5 integration tests for admin auth and blog multilingual"
  - "refactor: centralize validation code in admin_auth middleware"
  - "feat: add lang Tera function for view context"

### Step 7: Push to the remote
Run `git push` to push the commit to the remote repository.

### Step 8: Create a PR if none exists
Check if a PR already exists for the branch:
- Run `gh pr list --head <branch-name>` to check for existing PR
- If no PR exists, create one automatically:
  - Generate a title from the branch name (convert to title case, e.g., `test/add-phase5-integration-tests` → "Add phase5 integration tests")
  - Generate a body summarizing the changes: list the files changed and summarize the commit message
  - Use `gh pr create --title "<title>" --body "<body>"`

## Output

- Print the result of each step
- If any step fails, print the error and stop
- Print the commit SHA after a successful push
- Print the PR URL after creating a PR
