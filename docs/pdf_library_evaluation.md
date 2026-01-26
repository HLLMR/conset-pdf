# PDF Library Evaluation

Scope: compare candidate libraries for Conset PDF extraction with focus on deterministic output, bounding-box precision, and production stability.

## Summary Table

| Library | Install Complexity | Text Extraction Quality | BBox Support | Performance (300-page PDF) | Maintenance Status | License Compatibility |
| --- | --- | --- | --- | --- | --- | --- |
| pdfium-render (PDFium bindings) | Medium: requires native PDFium binaries; build scripts download per platform; works on Windows/macOS/Linux | High: good spacing and layout fidelity; handles ligatures reasonably; supports font fallbacks | Excellent: character and glyph positions via PDFium APIs; page and object matrices available | Fast: PDFium is C++; typically seconds for 300 pages on modern CPU | Active: PDFium maintained by Chromium team; Rust bindings maintained and updated | BSD-like upstream + bindings MIT/Apache; compatible with Apache-2.0 usage |
| pdf-extract (pure Rust) | Low: cargo-only; no native deps | Medium: basic text with spacing heuristics; can struggle with complex layouts | Limited: no per-character bbox; may expose token-level regions only | Moderate: pure Rust; slower than PDFium but predictable | Moderate activity: community-maintained; fewer contributors | MIT/Apache-2.0; compatible |
| lopdf (pure Rust, low-level) | Low: cargo-only; no native deps | Low-Medium: raw content extraction; spacing reconstruction is manual | Partial: low-level objects accessible; per-character bbox requires significant custom work | Slow-Moderate: parsing in Rust; custom extraction cost depends on our code | Low-Moderate activity; stable but slower update cadence | MIT license; compatible |

## Detailed Notes

### pdfium-render (PDFium bindings)
- Install: Requires platform PDFium binaries; build script can download. CI needs cached binaries or per-platform fetch.
- Text: High fidelity text extraction; preserves spacing and order better than heuristic parsers; handles rotated text and ligatures reasonably.
- BBoxes: Strong support; per-character/glyph positions and transformation matrices exposed; good for precise coordinates.
- Performance: Fast for large docs; C++ core; typically a few seconds for ~300 pages on modern hardware.
- Maintenance: PDFium is actively maintained by Chromium; Rust bindings have regular updates.
- License: PDFium (BSD-style); bindings are MIT/Apache; compatible with Apache-2.0 consumers.

### pdf-extract (pure Rust)
- Install: Pure Rust; no native deps.
- Text: Medium quality; spacing reconstruction heuristic; may mis-handle complex layouts or mixed writing modes.
- BBoxes: Limited; generally token-level, not per-character; may require patching to expose detailed positions.
- Performance: Moderate; acceptable for mid-sized docs but slower than PDFium on large, complex files.
- Maintenance: Community-maintained; moderate activity and issue backlog.
- License: MIT/Apache; compatible.

### lopdf (pure Rust, low-level)
- Install: Pure Rust; no native deps.
- Text: Raw content; spacing and ordering reconstruction left to consumer; higher engineering effort for quality output.
- BBoxes: Partial; objects accessible but per-character coordinates require custom text rendering logic; high effort.
- Performance: Parsing is fine; extraction performance depends on our own rendering/text logic; likely slower to reach production-grade quality.
- Maintenance: Stable but lower activity; slower cadence of updates.
- License: MIT; compatible.

## Recommendation
- Choose **pdfium-render** as the primary backend for production:
  - Deterministic output: PDFium’s mature parser yields consistent ordering and spacing.
  - BBox precision: Provides per-character/glyph coordinates with transforms, meeting our accuracy needs.
  - Production stability: Battle-tested in Chromium; bindings are maintained; low risk of segfaults compared with self-built parsers.
- Keep **pdf-extract** as a lightweight fallback for environments where native PDFium is not available; accept reduced layout fidelity and bbox detail.
- Use **lopdf** only for low-level PDF manipulation or debugging, not as the main extraction engine, given the engineering effort to reach precise bbox/text fidelity.
