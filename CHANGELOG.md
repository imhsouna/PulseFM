# Changelog

All notable changes to this project will be documented in this file.

## [0.1.10] - 2026-02-08

### Fixed
- Stop button now halts audio streams cleanly to prevent overlap after repeated start/stop.
- License tab layout now resizes cleanly with a side-by-side layout on wide screens.

## [0.1.9] - 2026-02-08

### Fixed
- Embedded LICENSE text in app so it always displays from /Applications.

## [0.1.8] - 2026-02-08

### Fixed
- Added missing `rand` dependency for random PI generator (cross‑platform builds).

## [0.1.7] - 2026-02-08

### Fixed
- License tab scrolling now fills available space.

## [0.1.6] - 2026-02-08

### Changed
- Updated macOS app icon to the lifeline logo.

## [0.1.5] - 2026-02-08

### Added
- macOS app bundle generation with icon (no Terminal popup when launching .app).

### Changed
- Removed terminal banner for clean UI-only launch.

## [0.1.4] - 2026-02-08

### Changed
- Release/CI matrix trimmed to macOS (arm64), Windows, and Linux to avoid Intel macOS runner delays.

## [0.1.3] - 2026-02-08

### Fixed
- Linux CI install step line continuations corrected to prevent apt failures.

## [0.1.2] - 2026-02-08

### Fixed
- Linux CI dependencies expanded and non-interactive installs for reliable builds.

## [0.1.1] - 2026-02-07

### Fixed
- GitHub Actions runners updated and Linux build dependencies added for release artifacts.

## [0.1.0] - 2026-02-07

### Added
- Modernized UI layout and styling.
- RadioDNS tab with full automation: SI.xml generation, logo creation/resizing, and DNS helper tools.
- One-click copy actions for FQDN, bearer, CNAME, SRV, and DNS bundle.
- File picker for logo source image and quick open of output folder/SI.xml.
- Validation of RadioDNS pack content and logo dimensions.
- Cross-platform GitHub Actions for macOS, Windows, and Linux releases.
- Local release build script.
- Terminal startup banner for quick usage guidance.

### Changed
- Readme updated for cross-platform build and release flow.

[0.1.10]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.10
[0.1.9]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.9
[0.1.8]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.8
[0.1.7]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.7
[0.1.6]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.6
[0.1.5]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.5
[0.1.4]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.4
[0.1.3]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.3
[0.1.2]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.2
[0.1.1]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.1
[0.1.0]: https://github.com/imhsouna/PulseFM/releases/tag/v0.1.0
