# Release

If not using nix, install cargo-dist and cargo-release:

`cargo install cargo-dist`
`cargo install cargo-release`

Update version to something like 1.0.9 in
- Cargo.toml
- CHANGELOG.md
- default.nix

git push changes on the main branch.

Cut the release (push tag and publish):

`cargo release 1.0.9`

If no errors, then

`cargo release 1.0.9 --execute`
