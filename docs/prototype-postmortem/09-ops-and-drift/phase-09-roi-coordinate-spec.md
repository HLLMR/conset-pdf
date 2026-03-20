# Phase 09 — ROI Coordinate Space Canonical Specification

**Status**: This is the canonical reference document. All future documents that discuss ROI coordinates must cite this file. Ambiguous wording in existing docs is resolved here.

---

## 1. Executive Summary

The prototype uses **three distinct coordinate spaces** that interact at specific boundary points. Each space has a different origin and unit. Getting this wrong produces undetectable ROI misconfiguration — the overlay draws in the correct screen position while the detection queries the wrong region of the PDF.

| Space | Origin | Unit | Used In |
|-------|--------|------|---------|
| **Profile/ROI Space** | Bottom-left | Normalized 0–1 | Layout profile JSON, `NormalizedROI` type |
| **Screen/Canvas Space** | Top-left | Pixels | GUI canvas, mouse events, `screenToNormalizedROI()` |
| **Text Item / Absolute Space** | Top-left | PDF points | `TextItemWithPosition`, `PageContext.getTextItems()` |
| **Transcript Span / Bbox Space** | Top-left | PDF points | `LayoutSpan.bbox`, `canonicalize.ts` |

The conversion path is:
```
Screen Space (pixels, TL) → Profile/ROI Space (normalized, BL) → Absolute Text Space (points, TL)
```

Two conversions happen:
1. **GUI → Profile**: `screenToNormalizedROI()` in `wizard-utils.js`
2. **Profile → Text Items**: `getTextItemsInROI()` in `pageContext.ts`

---

## 2. Profile / ROI Space (Canonical)

### 2.1 Definition

- **Origin**: Bottom-left corner of the page
- **Axis direction**: x increases rightward, y increases upward
- **Range**: All values normalized to 0.0–1.0
- **Applies to rotation**: After visual rotation normalization (see §5)
- **Field name in JSON**: `roiSpace: "visual"` (default; also the only value in use)

### 2.2 Canonical Source

`packages/core/src/layout/types.ts`:
```typescript
export interface NormalizedROI {
  /**
   * Normalized coordinates (0.0 - 1.0) relative to page
   * Origin: bottom-left (PDF standard)
   */
  x: number;      // Left edge (0.0 = left margin, 1.0 = right margin)
  y: number;      // Bottom edge (0.0 = bottom margin, 1.0 = top margin)
  width: number;  // Width as fraction of page width
  height: number; // Height as fraction of page height (extends UPWARD from y)
}
```

### 2.3 Visual Diagram (Landscape 24×36 Drawing)

```
(0,1)─────────────────────────────────────(1,1)
  │                                          │
  │         Page Content Area               │
  │                                          │
  │                          ┌────────────┐  │
  │                          │  sheetId   │  │
  │                          │  ROI       │  │
  │                          │ x=0.78     │  │
  │                          │ y=0.05     │  │
  │                          │ w=0.20     │  │
  │                          │ h=0.18     │  │
  │                          └────────────┘  │
(0,0)─────────────────────────────────────(1,0)
```

The title block is at the **bottom-right** in landscape drawings. In Profile Space, this means:
- x is near 1.0 (rightward)
- y is near 0.0 (bottom)
- width and height are small fractions

### 2.4 Common Title Block Region Examples

From `QUICK_START.md` (for 24×36 landscape drawings with bottom-right title block):

| Region | Typical x | Typical y | Typical width | Typical height |
|--------|-----------|-----------|---------------|----------------|
| Sheet Number | 0.75–0.85 | 0.0–0.15 | 0.15–0.25 | 0.15–0.25 |
| Sheet Title | 0.50–0.75 | 0.0–0.15 | 0.30–0.50 | 0.15–0.25 |

Note: y and height are measured **from the bottom**. A sheet number at the very bottom-right of a landscape page would have `y=0.0`, because the bottom of the ROI starts at the page bottom.

---

## 3. Screen / Canvas Space

### 3.1 Definition

- **Origin**: Top-left corner of the canvas element
- **Axis direction**: x increases rightward, y increases downward
- **Range**: Integer pixels, 0 to (canvas width-1) / (canvas height-1)
- **Used in**: Mouse event coordinates in `roiOverlayController.js`, `RoiOverlay` drawing

### 3.2 Conversion to Profile Space

`wizard-utils.js` function `screenToNormalizedROI(screenX, screenY, screenWidth, screenHeight, canvasWidth, canvasHeight)`:

```javascript
// Normalize to [0,1]
const normX = screenX / canvasWidth;
const normY = screenY / canvasHeight;
const normWidth = screenWidth / canvasWidth;
const normHeight = screenHeight / canvasHeight;

// Flip Y axis: screen top is y=0, profile space bottom-left has y=0 at bottom
// Profile y = 1 - (screenY_normalized + height_normalized)
const profileY = 1 - normY - normHeight;

return {
  x: clamp(normX, 0, 1),
  y: clamp(profileY, 0, 1),
  width: clamp(normWidth, 0, 1),
  height: clamp(normHeight, 0, 1),
};
```

**Integration test**: `tests/integration/roi-coordinates.test.ts` verifies this conversion with known test cases, including:
- Top-left screen corner → `y ≈ 0.9375` in profile space (high y value because it's near the screen top, which is near the page top, but the page top is y=1 in profile space)
- Bottom-right screen corner → `y ≈ 0.0` in profile space (bottom of page)

---

## 4. Absolute Text Item Space

### 4.1 Definition

- **Origin**: Top-left corner of the page
- **Axis direction**: x increases rightward, y increases downward
- **Range**: Float values in PDF points (1/72 inch); typical landscape page ≈ 1728 × 1152 points for 24×36 at 72dpi
- **Used in**: `TextItemWithPosition.x/y/width/height`, `PageContext.getTextItems()`

### 4.2 Conversion from Profile Space (getTextItemsInROI)

`packages/core/src/analyze/pageContext.ts`, `getTextItemsInROI()`:

```typescript
// ROI uses bottom-left origin (y: 0.0 = bottom, y: 1.0 = top) — PDF standard
// Text items use top-left origin (y: 0 = top, y = max = bottom)

const absX = expandedRoi.x * this._pageWidth;
// Convert bottom-left ROI to top-left text item coordinate:
// absY = height * (1 - roiY - roiHeight)
const absY = this._pageHeight * (1.0 - expandedRoi.y - expandedRoi.height);
const absWidth = expandedRoi.width * this._pageWidth;
const absHeight = expandedRoi.height * this._pageHeight;
```

This conversion is correct and has been verified by the integration tests.

---

## 5. Transcript Span / Bbox Space

### 5.1 Definition

- **Origin**: Top-left corner of the page (after rotation normalization)
- **Axis direction**: x increases rightward, y increases downward
- **Format**: `[x0, y0, x1, y1]` as a 4-tuple `[number, number, number, number]`
- **Used in**: `LayoutSpan.bbox`, output of `canonicalize.ts`

### 5.2 Rotation Normalization

`canonicalize.ts` normalizes all rotation to 0 by transforming bboxes:

| Original Rotation | Transform |
|------------------|-----------|
| 0° | No change |
| 90° (clockwise) | `[height-y1, x0, height-y0, x1]` (new page size: h×w) |
| 180° | `[width-x1, height-y1, width-x0, height-y0]` |
| 270° (clockwise) | `[y0, width-x1, y1, width-x0]` (new page size: h×w) |

After normalization, all spans are in the visual orientation the user sees. This is why `roiSpace: "visual"` was chosen as the default profile space — both the ROI and the canonicalized spans operate in the same post-rotation visual space.

### 5.3 Stable Sort Order

After normalization, spans are sorted:
1. Primary: `y0` ascending (top to bottom)
2. Secondary: `x0` ascending (left to right)
3. Tie-break: original order (stable sort)

Floating-point tolerance: 0.1 points.

---

## 6. The `roiSpace: "visual"` vs `"pdf"` Distinction

| Value | Meaning |
|-------|---------|
| `"visual"` | ROI coordinates are in the *visually rendered* space after rotation is applied. This is the default and the only value used in practice. |
| `"pdf"` | ROI coordinates are in the raw PDF coordinate space before rotation. Reserved for future use; no implementation handles this case currently. |

**Practical implication**: All layout profiles in the inventory use `roiSpace: "visual"`. Any future profile that uses `roiSpace: "pdf"` will silently receive wrong results because `getTextItemsInROI` does not handle the `"pdf"` case — it always operates in visual space. The `layout/load.ts` sets `roiSpace = 'visual'` as default if absent, but does not implement the `"pdf"` conversion path.

**Rust implication**: Do not implement `roiSpace: "pdf"` unless the extraction pipeline is updated to perform the inverse visual-rotation transform before ROI filtering. Until then, only `"visual"` should be accepted.

---

## 7. End-to-End Coordinate Flow

```
User draws ROI on canvas
  ↓ (screen pixels, top-left origin)
screenToNormalizedROI()
  ↓ (normalized 0..1, bottom-left origin)
Profile JSON saved (NormalizedROI)
  ↓
getTextItemsInROI(roi)
  ↓ (converts to absolute points, top-left origin)
TextItemWithPosition[] filtered by ROI
  ↓
Text assembled, regex applied → sheet ID detected
```

---

## 8. Migration Tests / Regression Guards

The following tests must not be removed. They are the coordinate-space regression guardrails:

| Test File | What It Guards |
|-----------|---------------|
| `tests/integration/roi-coordinates.test.ts` | `screenToNormalizedROI()` conversion for known inputs (top-left to bottom-left) |
| `tests/transcript/` (canonicalization tests) | Rotation normalization produces correct bbox transform for 90°/180°/270° |

**Required addition** (not yet implemented): A test that:
1. Creates a layout profile with a known ROI
2. Creates a synthetic page with a text item at a known position
3. Asserts that `getTextItemsInROI()` returns the text item when the ROI overlaps it
4. Asserts that `getTextItemsInROI()` does not return the text item when the ROI is disjoint

This test would catch regressions in the bottom-left to top-left coordinate conversion.

---

## 9. Known Ambiguous Wording in Existing Docs (Resolved Here)

| Location | Ambiguous Phrase | Resolution |
|---------|-----------------|------------|
| `QUICK_START.md` | "Origin: Bottom-left corner" | Correct — this describes Profile/ROI Space |
| `canonicalize.ts` comment | "y=0 at top, y increases downward" | Correct — this describes Transcript Span Space (after canonicalization) |
| `layout-template.json` | `"roiSpace": "visual"` | This indicates ROI is in post-rotation visual space, not raw PDF space |
| `pageContext.ts` comment | "ROI uses bottom-left origin (PDF standard)" | Correct — Profile/ROI Space IS bottom-left; text items are then converted |
| Phase 2 transcript doc | "top-left origin" | Correct for transcript bboxes specifically; does NOT apply to ROI profile coordinates |

**The rule**: When you see "bottom-left origin" in these docs, it refers to Profile/ROI Space. When you see "top-left origin," it refers to either Screen Space, Text Item Space, or Transcript Span Space. These are not in conflict — they are different spaces with a defined conversion between them.

---

## 10. Rust Implementation Requirements

| Requirement | Rationale |
|-------------|-----------|
| Rust profile parser must validate ROI coordinates as bottom-left origin | Incorrect origin assumption produces inverted Y detection |
| Rust extraction layer must implement the same Y-flip conversion as `getTextItemsInROI` | `absY = pageHeight * (1.0 - roi.y - roi.height)` |
| Rust rotation normalization must match the 90°/180°/270° transforms in `canonicalize.ts` exactly | Any deviation produces coordinate mismatch between ROI and text items |
| Only `roiSpace: "visual"` should be accepted until `"pdf"` path is implemented | Prevents silent wrong-space detection |
| Integration test for the coordinate chain is required before first release | Coordinate bugs are silent and hard to catch in production |
| Canvas-to-profile conversion in Tauri frontend must match `screenToNormalizedROI` exactly | Profile saved from GUI must be valid for CLI and vice versa |
