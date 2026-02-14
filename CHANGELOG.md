# Changelog

## [0.2.1] - 2026-02-14

Fix Package Metadata

### Fixed

- Updated `Cargo.toml` with correct package metadata, including name, version, description, and repository URL.

## [0.2.0] - 2026-02-14

### Added

- Comprehensive unit tests for core modules.
- Full API documentation with examples.
- CI/CD workflows for automated testing and releases.
- Cross-platform support for Linux, Windows, and macOS.

### Changed

- Reorganized project structure into `core`, `presentation`, and `ui` modules.
- Extracted business logic into a reusable library.
- Simplified `main.rs` by extracting event handlers.
- Centralized assets into `assets/icons/`.

### Removed

- Legacy WiX packaging files in favor of `cargo-dist`.
