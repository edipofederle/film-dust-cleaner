# Film Scanning & Restoration — Research Notes

Techniques and algorithms used by professional film labs. Items marked **implemented** are
already in the tool. The rest are candidates for future work.

---

## Dust & Scratch Removal

### Local contrast detection — **implemented**
Estimate a per-pixel background via Gaussian blur; flag pixels significantly brighter than
their neighbourhood as scratches. Drive TELEA inpainting from that mask.
Good for bright scratches on dark-background scans. Parameters: sigma, threshold, inpaint radius.

### Wet gate scanning — hardware only
The film passes through a liquid gate whose refractive index matches the film base, optically
filling scratches and making them invisible to the sensor. Cannot be replicated in software —
it is a physical optics technique. Requires perchloroethylene or isopropanol in specialised hardware
(DFT Scanity, ARRISCAN XT).

### Digital ICE / infrared channel detection — requires hardware IR pass
Scanners (Nikon Coolscans, Plustek) fire a second infrared pass. Film emulsion is IR-transparent;
dust and scratches block IR and appear as bright spots. The IR channel becomes the inpaint mask.
The inpainting step itself is pure software (same TELEA or NS algorithm we use), but the accurate
mask generation requires the IR-equipped scanner hardware.
**Limitation**: does not work on B&W film or Kodachrome — metallic silver grains are IR-opaque
and are mis-detected as defects.
**Software equivalent**: SilverFast iSRD (needs compatible scanner).

### CNN / GAN-based defect detection — future candidate
Train a convolutional network on pairs of clean/dirty scans to learn a defect mask without
needing an IR channel. DustNet (DigMyPics) is a commercial example trained on millions of
annotated particles.
Published baseline: *A Generative Adversarial Approach with Residual Learning for Dust and
Scratches Artifacts Removal* — arXiv 2009.10663.
Would replace the Gaussian-blur heuristic mask with a learned one, improving detection of
low-contrast and curved scratches.

---

## Grain Reduction

### NLMeans denoising — **implemented**
OpenCV `fastNlMeansDenoising` (grayscale) / `fastNlMeansDenoisingColored` (colour).
Two-stage: find featureless patches to profile grain frequency/amplitude, then filter with
a patch-similarity kernel. Parameter `h` controls strength (3–15 typical for film grain).

### BM3D — future candidate
Block-Matching 3D denoising. State-of-the-art for film grain: groups similar patches across
the image into 3D stacks, applies a collaborative Wiener filter in the transform domain.
Reference implementation: `bm3d` Python package (Marc Lebrun). Higher quality than NLMeans
but significantly slower. Would be a drop-in upgrade for the denoise step.

### Neural denoisers (DnCNN, FFDNet) — future candidate
CNNs trained specifically on film grain patterns. FFDNet is spatially adaptive — it accepts
a noise-level map as input, allowing different denoising strengths per region.
Useful for scans where grain is uneven (e.g. push-processed film in shadows vs. highlights).

---

## Negative Inversion & Colour Correction

### Bitwise invert + per-channel auto-levels — **implemented**
Simple pipeline: bitwise NOT → per-channel NORM_MINMAX normalisation to neutralise the
orange mask. Works well for a quick positive from a colour negative.

### Logarithmic inversion with sampled film base — future candidate
More accurate approach used by tools like Negmaster and Filmomat SmartConvert:
1. Sample the unexposed film border to measure the orange mask baseline per channel.
2. Per channel: `positive = log(base) − log(pixel)`, then normalise.
3. Apply a tone curve modelled on the response of photochemical paper (Fuji Frontier / Noritsu).
Produces more accurate skin tones and shadow separation than the simple per-channel stretch.

### Per-film-stock LUTs — future candidate
Professional scanners (Fuji Frontier, Noritsu LS-600) ship with calibrated LUTs for each film
emulsion (Kodak Portra 400, Fuji 400H, etc.) that account for each stock's unique dye response
curves. Building or sourcing open LUTs per stock would allow film-accurate conversions.
Reference: *Negative Lab Pro* profiles, *Grain2Pixel* open LUT project.

---

## Dynamic Range

### 16-bit capture — best practice
High-end drum scanners (Hasselblad FlexTight X5, Howtek) scan at 16 bits per channel with
Dmax up to 4.0, preserving shadow detail that 8-bit scans clip. Implementing 16-bit I/O in
the pipeline (OpenCV supports `CV_16U`) would be a straightforward upgrade.

### Multi-exposure HDR merging — future candidate
Scan at multiple exposures (like bracketed HDR photography) and merge to recover both
highlights and deep shadows. Relevant for very high-contrast film (slide/reversal film).
OpenCV `MergeDebevec` / `MergeRobertson` implement this. Requires the scanner to support
multiple-exposure passes (drum scanners and some flatbeds do).

---

## Sharpening

### Unsharp mask — basic
Standard high-pass boost. Amplifies grain alongside detail; not ideal for film scans.
Already available via `GaussianBlur` + `addWeighted` in OpenCV.

### Deconvolution sharpening — future candidate
Models the scanner's optical blur as a point spread function (PSF) and mathematically reverses
it using Wiener or Richardson-Lucy deconvolution. Recovers detail without amplifying grain
because it targets the specific blur kernel rather than all high frequencies.
OpenCV has no built-in deconvolution; would need a custom implementation or `scipy.signal.wiener`.
Commercial tools: Piccure+, DxO Smart Lighting.

---

## References

- Telea, A. (2004). An Image Inpainting Technique Based on the Fast Marching Method.
  *Journal of Graphics Tools*, 9(1), 23–34. https://doi.org/10.1080/10867651.2004.10487596
- Dabov, K. et al. (2007). Image Denoising by Sparse 3D Transform-Domain Collaborative Filtering (BM3D).
  *IEEE Transactions on Image Processing*, 16(8).
- Nair, S. et al. (2020). A Generative Adversarial Approach with Residual Learning for Dust and
  Scratches Artifacts Removal. arXiv:2009.10663.
- Digital ICE — Wikipedia. https://en.wikipedia.org/wiki/Digital_ICE
- Wet-transfer film gate — Wikipedia. https://en.wikipedia.org/wiki/Wet-transfer_film_gate
- SilverFast iSRD. https://www.silverfast.com
- Filmomat SmartConvert. https://www.filmomat.eu/smartconvert
