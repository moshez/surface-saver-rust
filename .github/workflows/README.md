# GitHub Workflows

This directory contains GitHub Actions workflows for continuous integration and release management.

## CI Workflow (`ci.yml`)

Runs on all pull requests and pushes to the `trunk` branch.

### Jobs:

1. **Test** - Runs tests on stable, beta, and nightly Rust
2. **Rustfmt** - Checks code formatting
3. **Clippy** - Runs the Rust linter
4. **Coverage** - Generates code coverage reports (uploaded to Codecov)
5. **Security Audit** - Checks for known vulnerabilities in dependencies

## Release Workflow (`release.yml`)

Triggered when a new version tag is pushed (e.g., `v1.0.0`).

### Features:

- Creates a GitHub release
- Builds binaries for multiple platforms:
  - Linux (x86_64, aarch64)
  - Windows (x86_64)
  - macOS (x86_64, aarch64)
- Uploads binaries as release assets

## Usage

To trigger a release:
```bash
git tag v1.0.0
git push origin v1.0.0
```