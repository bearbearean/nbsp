# Development

## Development environment

*TODO: Write out details for the development environment.*

## Version release procedure

[A GitHub Actions workflow](https://github.com/bearbearean/nbsp/actions/workflows/release-plz.yml) using [release-plz](https://release-plz.dev) is set up to automatically create [the CHANGELOG.md](https://github.com/bearbearean/nbsp/blob/main/CHANGELOG.md) and [a release PR](https://github.com/bearbearean/nbsp/issues?q=is%3Apr%20chore%3A%20release) whenever new commits are pushed to the `main` branch.

The release PR stays open until we want to release a new version, any changes to `main` will be added in the PR. When we want to release a new version, all we have to do is merge the PR and the actions workflows will take care of the rest.

Release-plz will publish [the nbsp crate](https://crates.io/crates/nbsp) (using trusted publishing) and create [a release on GitHub](https://github.com/bearbearean/nbsp/releases).

Once a new release is created on GitHub [the release binary workflow](https://github.com/bearbearean/nbsp/actions/workflows/release-binary.yml) will then compile the nbsp binary and upload it to the GitHub release.
