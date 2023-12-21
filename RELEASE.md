# Release

Insall cargo-dist and cargo-release:

`cargo binstall cargo-dist`
`cargo install cargo-release`

Update version to something like 1.0.5 in Cargo.toml and CHANGELOG.md.

git push changes on the main branch.

Cut the release (push tag and publish):

`cargo release 1.0.5`
