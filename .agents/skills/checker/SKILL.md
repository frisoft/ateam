---
name: checker
description: Runs the same CI checks as the GitHub CI workflow locally. Use this skill before pushing commits to verify the code passes all CI checks.
---

# Checker Skill

This skill runs the same checks as the GitHub CI workflow.

## Description

Reads the GitHub CI workflow file and executes exactly the same checks locally.

## When to use

Use this skill before pushing commits to verify the code passes all CI checks locally.

## Implementation

When this skill is invoked, perform the following steps:

### Step 1: Read the CI workflow
Read `.github/workflows/ci.yaml` to identify the checks to run.

### Step 2: Execute the CI checks
Run exactly the same commands defined in the workflow file.

If any check fails, fix the issues and rerun until all checks pass.

## Output

- Print results of each check
- If any check fails, print the failure details
