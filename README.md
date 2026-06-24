# film-dust-cleaner

Removes dust, scratches, and grain from scanned film photos. Optionally inverts colour negatives to positives. Runs as a CLI tool or a local web UI.

## How it works

Rather than a global brightness threshold, the tool estimates a local background for each pixel using Gaussian blur and flags pixels that are significantly brighter than their surroundings. Those are inpainted using the TELEA algorithm. An optional NLMeans pass reduces film grain after inpainting.

This approach avoids false positives on naturally bright areas (skin highlights, white clothing) while catching thin scratches and dust spots of varying intensity.

## Requirements

- Rust (stable)
- OpenCV 4.x (`brew install opencv` on macOS)

## Build

```bash
cargo build --release
```

The binary will be at `target/release/film-dust-cleaner`.

## Commands

### `clean` — remove dust, scratches, and grain

```bash
film-dust-cleaner clean <input> <output> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--sigma` | `15.0` | Blur radius for background estimation. Larger values catch wider scratches. |
| `--threshold` | `30.0` | How much brighter than surroundings a pixel must be to be flagged. Lower = more aggressive. |
| `--inpaint-radius` | `5.0` | Neighbourhood radius used during inpainting. Increase for thicker scratches. |
| `--denoise` | `0.0` | Grain reduction strength. `0` = disabled; `3–15` typical range. |
| `--invert` | `false` | Treat input as a colour negative and invert before cleaning. |
| `--exposure` | `0.0` | Exposure adjustment in EV stops (-2.0 to +2.0). Applied after cleaning. |
| `--contrast` | `1.0` | Contrast multiplier (1.0 = no change, >1 increases contrast). Applied after cleaning. |

```bash
# Default — good for heavily scratched scans
film-dust-cleaner clean scan.jpg cleaned.jpg

# Conservative — safer for cleaner scans
film-dust-cleaner clean scan.jpg cleaned.jpg --threshold 50

# Aggressive with grain reduction
film-dust-cleaner clean scan.jpg cleaned.jpg --threshold 20 --sigma 20 --denoise 8

# Brighten and boost contrast
film-dust-cleaner clean scan.jpg cleaned.jpg --exposure 0.5 --contrast 1.3

# Colour negative
film-dust-cleaner clean neg.jpg positive.jpg --invert
```

### `invert` — convert a colour negative to a positive

```bash
film-dust-cleaner invert <input> <output>
```

Performs bitwise inversion followed by per-channel auto-levels to neutralise the orange mask.

### `serve` — start the web UI

```bash
film-dust-cleaner serve [--port 3000]
```

Opens a browser UI at `http://localhost:<port>` with live before/after preview and real-time slider adjustment.

## Tuning guide

- If scratches remain → lower `--threshold` or raise `--sigma`.
- If fine detail is being smoothed out → raise `--threshold`.
- If inpainted areas look patchy → increase `--inpaint-radius`.
- If grain is distracting → raise `--denoise` (start at `5`, stop before it looks plasticky).

## Research

`research.md` documents additional techniques used by professional labs (BM3D denoising, CNN-based defect detection, log-space negative inversion, per-film-stock LUTs, multi-exposure HDR, deconvolution sharpening) that are candidates for future implementation.

## References

- Telea, A. (2004). [An Image Inpainting Technique Based on the Fast Marching Method](https://doi.org/10.1080/10867651.2004.10487596). *Journal of Graphics Tools*, 9(1), 23–34.
