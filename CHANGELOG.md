# Changelog

## v1.0.9

- Make it a regular nix package.

## v1.0.8

- Upgrade crates.

## v1.0.7

- Release binary for aarch64-apple-darwin as well.

## v1.0.6

- Upgrade crates.

## v1.0.5

- Replace unmaintained crate dotenv with dotenvy.
- Upgrade crates.
- Add Audit.

## v1.0.4

- Improve error messages.
- Improve help messages.
- Upgrade crates.

## v1.0.3

- Add --version option.

## v1.0.2

- Fix new line missing in reports.
- Upgrade crates.
- Start using cargo-dist to distribute a-team.

## v1.0.1

- Replace failure crate with anyhow.
- Make it a library.
- Upgrade crates.

## v1.0.0

- User async/await with tokio instead of rayon. Improved performance when using --blame.
- Upgrade crates.

## v0.8.5

- --query option can be used multiple times.
- Replace structopt crate with clap.

## v0.8.4

- Report an error when team members are needed but `read:org` scope is not in the GitHub token to get them.

## v0.8.3

- Upgrade to Rust edition 2021.
- Add --requested option.
- Add the Requested (explicitly) score.
- Simplify request query by removing -author: and -reviewed-by: conditions to avoid GH bug to return an empty result.
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
