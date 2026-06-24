# TODO

Tracked implementation ideas and known tasks. See `research.md` for deeper background on
professional lab techniques.

---

## Drop OpenCV dependency (pure Rust)

The goal is a self-contained binary with no native C++ dependency. Each OpenCV call below
can be replaced in pure Rust. Everything except TELEA inpainting is straightforward.

### Trivial — pure pixel arithmetic

- [ ] `core::subtract` → manual per-pixel subtraction with saturation
- [ ] `imgproc::threshold` → simple pixel comparison loop
- [ ] `core::bitwise_not` → `255 - pixel` per channel
- [ ] `core::split` / `core::merge` → iterate channels manually
- [ ] `core::normalize` (NORM_MINMAX) → find min/max, rescale linearly
- [ ] `imgproc::cvt_color` BGR→Gray → weighted average: `0.114·B + 0.587·G + 0.299·R`

### Easy — standard algorithms

- [ ] `imgproc::gaussian_blur` → separable 1D convolution applied horizontally then
  vertically; or use the `imageproc` crate which already provides this
- [ ] `imgproc::dilate` (3×3 rect kernel) → 3×3 max filter over each pixel

### Medium — more involved but well-documented

- [ ] `photo::fast_nl_means_denoising` (NLMeans) → patch similarity search + weighted
  average; naive implementation is ~150 lines, efficient version needs integral images
  or summed-area tables for speed on large scans

### Hard — the real blocker

- [ ] `photo::inpaint` TELEA → requires implementing the Fast Marching Method
  (priority-queue wavefront propagation) followed by per-pixel weighted reconstruction
  from the advancing boundary. Subtle numerical details from the original paper affect
  output quality. Estimated effort: ~1 week. Alternative: substitute a simpler diffusion
  fill (lower quality but removes the hard dependency).

### File I/O

- [ ] Replace `imgcodecs::imread` / `imgcodecs::imwrite` with the `image` crate
  (`image::open`, `image::save_buffer`), which handles JPEG and PNG natively in Rust.

---

## From research.md — future processing features

- [ ] BM3D denoising — higher quality grain reduction than NLMeans; no Rust crate exists,
  would need a custom implementation
- [ ] CNN/GAN-based defect detection — replace the Gaussian heuristic mask with a learned
  one (reference: arXiv 2009.10663); catches low-contrast and curved scratches the current
  approach misses
- [ ] Log-space negative inversion with sampled film base — more accurate than the current
  bitwise NOT + auto-levels; sample the unexposed border per channel as the orange mask
  baseline
- [ ] Per-film-stock LUTs — calibrated inversion curves for specific emulsions
  (Kodak Portra, Fuji 400H, etc.)
- [ ] Multi-exposure HDR merging — scan at multiple exposures and merge for higher dynamic
  range; relevant for slide/reversal film
- [ ] Deconvolution sharpening — model and reverse the scanner's optical blur (PSF) using
  Wiener or Richardson-Lucy deconvolution; recovers detail without amplifying grain
- [ ] 16-bit pipeline — OpenCV supports `CV_16U`; preserves shadow detail that 8-bit clips
