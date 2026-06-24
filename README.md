# film-dust-cleaner

Removes dust and scratches from scanned film photos using local contrast detection and inpainting.

## How it works

Rather than a global brightness threshold, the tool estimates a local background for each pixel using Gaussian blur and flags pixels that are significantly brighter than their surroundings. Those are inpainted using the TELEA algorithm.

This approach avoids false positives on naturally bright areas (skin highlights, white clothing) while catching thin scratches and dust spots of varying intensity.

## Requirements

- Rust (stable)
- OpenCV 4.x (`brew install opencv` on macOS)

## Build

```bash
cargo build --release
```

The binary will be at `target/release/film-dust-cleaner`.

## Usage

```bash
film-dust-cleaner <input> <output> [OPTIONS]
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--sigma` | `15.0` | Blur radius for background estimation. Larger values catch wider scratches. |
| `--threshold` | `30.0` | How much brighter than surroundings a pixel must be to be flagged. Lower = more aggressive. |
| `--inpaint-radius` | `5` | Neighbourhood radius used during inpainting. Increase for thicker scratches. |

### Examples

```bash
# Default settings — good for heavily scratched scans
film-dust-cleaner scan.jpg cleaned.jpg

# Conservative — safer for cleaner scans
film-dust-cleaner scan.jpg cleaned.jpg --threshold 50

# Aggressive — for heavy or wide scratches
film-dust-cleaner scan.jpg cleaned.jpg --threshold 20 --sigma 20 --inpaint-radius 7
```

## References

- Telea, A. (2004). [An Image Inpainting Technique Based on the Fast Marching Method](https://doi.org/10.1080/10867651.2004.10487596). *Journal of Graphics Tools*, 9(1), 23–34.

## Tuning guide

- Start with defaults and inspect the result.
- If scratches remain, lower `--threshold` or raise `--sigma`.
- If fine detail is being smoothed out, raise `--threshold`.
- If inpainted areas look patchy, increase `--inpaint-radius`.
