# Changelog

All notable changes to this project will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.7.0] — 2026-06-24

### Added
- Multi-file support in the web UI: drop or select multiple scans at once
- Navigation bar with ← / → arrows and a `current / total` counter
- Keyboard navigation with arrow keys (← / →) when focus is not on a slider
- Per-photo result cache — revisiting a previously processed photo shows the cached result instantly
- Photos are sorted alphabetically on load

---

## [0.6.0] — 2026-06-24

### Added
- Download button in the web UI — appears after processing and saves the result as `<original_name>_cleaned.jpg`

---

## [0.5.0] — 2026-06-24

### Added
- Configuration file support (`film-dust-cleaner.toml` in current directory, or
  `~/.config/film-dust-cleaner/config.toml` as user-level config)
- `output_dir` config key: when set, the CLI `output` argument becomes optional and
  cleaned files are saved as `<output_dir>/<input_filename>`

---

## [0.4.0] — 2026-06-24

### Added
- Exposure adjustment (`--exposure`, EV stops, -2.0 to +2.0) via CLI and web UI slider
- Contrast adjustment (`--contrast`, multiplier, 0.5 to 3.0) via CLI and web UI slider
- Both adjustments are applied as a single linear transform after the full clean pipeline

---

## [0.3.0] — 2026-06-24

### Added
- Grain reduction via `fastNlMeansDenoising` (`--denoise` flag, slider in web UI)
- Negative inversion: bitwise NOT + per-channel auto-levels to neutralise the orange mask (`--invert` flag, toggle in web UI)
- New `invert` CLI subcommand for standalone negative-to-positive conversion
- `research.md` documenting professional lab techniques and future implementation candidates

### Changed
- CLI restructured into subcommands: `clean`, `invert`, `serve`

---

## [0.2.0] — 2026-06-24

### Added
- Local web UI served by an Axum HTTP server (`serve` subcommand)
- Before/after split view with sidebar controls
- Real-time parameter adjustment: sliders debounce at 400 ms and cancel in-flight requests

### Changed
- Core logic extracted into `lib.rs` for reuse across CLI and server

---

## [0.1.0] — 2026-06-24

### Added
- Dust and scratch removal using local contrast detection (Gaussian background estimation + TELEA inpainting)
- CLI with configurable `--sigma`, `--threshold`, and `--inpaint-radius`
- Rust port of original Python/OpenCV prototype
