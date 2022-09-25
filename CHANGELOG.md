# Changelog

## v0.8.5

- --query option can be used multiple times.
- Replace structopt crate with clap.

## v0.8.4

- Report an error when team members are needed but `read:org` scope is not in the GitHub token to get them.

## v0.8.3

- Upgrade to Rust edition 2021.
- Add --requested option.
- Add the Requested (explicitly) score.
- Simplify request query removing -author: and -reviewed-by: conditions to avoid GH bug to return an empty result.
- Improve parsing of code owner team members.

## v0.8.2

- Enable --include-reviewed-by-me when using --only-mine.

## v0.8.1

- Upgrade crates.

## v0.8.0

- Add --exclude-tests-none flag. Exclude pull requests without tests.
- Remove --include-tests-none flag. It is now the default.
- PR without CI tests are considered successful for the score.

## v0.7.1

- Add --json flag.
- Add --user option.
- Add --batch-size option.
- Add --include-drafts flag.
- Fix calculation of number of review and exclude author.
- Fix calculation of number of approvals
