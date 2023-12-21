# Release

Insall cargo-dist and cargo-release:

`cargo install cargo-dist`
`cargo install cargo-release`

Update version to something like 1.0.5 in Cargo.toml and CHANGELOG.md.

git push changes on the main branch.

Cut the release (push tag and publish):

`cargo release 1.0.5`

If no errors, then

`cargo release 1.0.5 --execute`
