# Changelog

## [0.2.2] - 2026-02-17

### Added

- Theme toggle button (System / Light / Dark) with persistence via `confy`.
- `ThemeMode` enum and theme management methods (`load_theme_config`, `save_theme_config`, `theme_index`) in `AppState`.
- `setup_theme_handlers` and `apply_theme_to_ui` functions in handlers module.
- Slint `Palette` integration for runtime color scheme switching.

### Changed

- Switched rendering backend from `renderer-software` to `renderer-femtovg` for improved performance.
- Pinned dependency versions (`slint` 1.15, `image` 0.25, `anyhow` 1, `arboard` 3, `rfd` 0.17) instead of using wildcard `*`.
- Added `serde` and `confy` dependencies for theme config serialization.
- Simplified platform-specific dependency configuration in `Cargo.toml`.
- Image display area now uses `Palette` theme colors instead of hardcoded values.

### Fixed

- Added comments in `.goreleaser.yaml` clarifying why macOS targets are disabled (GUI application, no cross-compilation support).

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
