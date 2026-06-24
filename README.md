# film-dust-cleaner

A film scan restoration tool that removes dust, scratches, and grain from scanned analog photos. Supports colour negative inversion, exposure and contrast adjustments, and runs as either a CLI tool or a local web UI with live before/after preview.

## How it works

Rather than a global brightness threshold, the tool estimates a local background for each pixel via Gaussian blur and flags pixels significantly brighter than their surroundings as defects. Those are repaired using the TELEA inpainting algorithm. An optional NLMeans pass reduces film grain, and a final linear transform applies exposure and contrast adjustments.

This approach avoids false positives on naturally bright areas (skin highlights, white clothing) while catching thin scratches and dust spots of varying intensity.

## Requirements

- Rust (stable)
- OpenCV 4.x

```bash
brew install opencv   # macOS
```

## Build

```bash
cargo build --release
```

Binary: `target/release/film-dust-cleaner`

---

## Commands

### `clean`

Remove dust, scratches, and optionally grain from a scan.

```bash
film-dust-cleaner clean <input> [output] [OPTIONS]
```

The `output` argument is optional when `output_dir` is set in the config file.

| Flag | Default | Description |
|------|---------|-------------|
| `--sigma` | `15.0` | Blur radius for background estimation. Larger values catch wider scratches. |
| `--threshold` | `30.0` | Brightness excess above background to flag as a defect. Lower = more aggressive. |
| `--inpaint-radius` | `5.0` | Neighbourhood radius used during inpainting. Increase for thicker scratches. |
| `--denoise` | `0.0` | Grain reduction strength (`0` = off, `3–15` typical). |
| `--invert` | `false` | Treat input as a colour negative and invert before cleaning. |
| `--exposure` | `0.0` | Exposure in EV stops (−2.0 to +2.0). Applied after cleaning. |
| `--contrast` | `1.0` | Contrast multiplier (1.0 = unchanged, >1 increases contrast). Applied after cleaning. |

**Examples**

```bash
# Defaults — good starting point for heavily scratched scans
film-dust-cleaner clean scan.jpg cleaned.jpg

# Conservative — safer for cleaner scans with less damage
film-dust-cleaner clean scan.jpg cleaned.jpg --threshold 50

# Aggressive dust removal with grain reduction
film-dust-cleaner clean scan.jpg cleaned.jpg --threshold 20 --sigma 20 --denoise 8

# Colour negative: invert, clean, brighten slightly
film-dust-cleaner clean neg.jpg positive.jpg --invert --exposure 0.3

# Adjust exposure and contrast only (set threshold high to skip inpainting)
film-dust-cleaner clean scan.jpg out.jpg --threshold 80 --exposure 0.5 --contrast 1.3
```

---

### `invert`

Convert a colour negative scan to a positive without dust removal.

```bash
film-dust-cleaner invert <input> <output>
```

Performs a bitwise inversion followed by per-channel auto-levels to neutralise the orange mask.

---

### `serve`

Start a local web UI with live before/after preview and real-time controls.

```bash
film-dust-cleaner serve [--port 3000]
```

Open `http://localhost:3000` in your browser. All parameters are available as sliders and update the result automatically.

---

## Configuration

Config is loaded from the first file found:

| Path | Scope |
|------|-------|
| `./film-dust-cleaner.toml` | Project-local (current directory) |
| `~/.config/film-dust-cleaner/config.toml` | User-level |

```toml
# Example config
output_dir = "~/Pictures/cleaned"
```

With `output_dir` set, the `output` argument to `clean` is optional — files are saved as `<output_dir>/<input_filename>`.

---

## Tuning guide

| Symptom | Fix |
|---------|-----|
| Scratches still visible | Lower `--threshold` or raise `--sigma` |
| Fine detail being erased | Raise `--threshold` |
| Inpainted areas look patchy | Increase `--inpaint-radius` |
| Too much grain | Raise `--denoise` (start at `5`, stop before it looks plastic) |
| Image too dark/bright | Adjust `--exposure` |
| Image looks flat | Raise `--contrast` above `1.0` |

---

## Further reading

- `research.md` — professional lab techniques and candidates for future implementation (BM3D, CNN-based defect detection, log-space inversion, per-film-stock LUTs, HDR merging, deconvolution sharpening)
- `TODO.md` — tracked implementation tasks including a plan to drop the OpenCV dependency

## References

- Telea, A. (2004). [An Image Inpainting Technique Based on the Fast Marching Method](https://doi.org/10.1080/10867651.2004.10487596). *Journal of Graphics Tools*, 9(1), 23–34.
