# Quick Start Guide

**Last verified**: 2026-03-01

This guide walks you through the canonical ROI-first workflow for using `conset-pdf`.

## Canonical Workflow

### Step 1: Preview Detection

Test your layout profile on sample pages:

```bash
conset-pdf detect \
  --input your-drawing-set.pdf \
  --layout layouts/my-project-layout.json \
  --pages 1,5,10,20,50 \
  --output-preview preview.json \
  --verbose
```

**What to look for:**
- Success rate should be ≥80%
- Check that detected IDs match what you see in the PDF
- Review warnings for low-confidence detections

### Step 2: Adjust Layout Profile

If success rate is low, adjust ROI coordinates in your layout profile:

```bash
# Edit the layout profile
# Adjust x, y, width, height values (normalized 0-1)
```

**Common ROI Values** (for 24"×36" landscape drawings with bottom-right title block):

**Sheet Number:**
- `x: 0.75-0.85`, `y: 0.0-0.15`, `width: 0.15-0.25`, `height: 0.15-0.25`

**Sheet Title:**
- `x: 0.50-0.75`, `y: 0.0-0.15`, `width: 0.30-0.50`, `height: 0.15-0.25`

**ROI Coordinate System:**
- Origin: Bottom-left corner
- All values: 0.0 to 1.0 (normalized)
- `x`: Distance from left edge (0.0 = left, 1.0 = right)
- `y`: Distance from bottom edge (0.0 = bottom, 1.0 = top)

### Step 3: Merge Addenda

Once detection is working well, run the merge:

```bash
conset-pdf merge-addenda \
  --original source-pdfs/original/IFC\ Set_FULL_2025-10-01.pdf \
  --addenda source-pdfs/addenda/Add4\ Set_FULL_2025-10-20.pdf \
  --output conset-output/merged/IFC+Add4.pdf \
  --type drawings \
  --layout layouts/my-project-layout.json \
  --bookmark \
  --report conset-output/reports/merge-add4-report.json \
  --inventory-dir conset-output/inventories \
  --verbose
```

**Expected Output:**
- Merged PDF with replaced and inserted pages in correct order
- Bookmarks regenerated from detected sheet IDs and titles (if `--bookmark` enabled)
- JSON report with merge statistics and warnings
- Inventory files showing detected IDs per page

## Creating a Layout Profile

### Option A: Copy the Template

```bash
cp layouts/layout-template.json layouts/my-project-layout.json
# Edit the file with your ROI coordinates
```

### Option B: Use Inline ROI (Quick Test)

```bash
conset-pdf detect \
  --input your-file.pdf \
  --sheet-id-roi "0.78,0.05,0.20,0.18" \
  --sheet-title-roi "0.55,0.05,0.40,0.18" \
  --pages 1,5,10 \
  --verbose
```

### Example Layout Profile

```json
{
  "name": "project-xyz-titleblock",
  "version": "1.0.0",
  "description": "Layout for Project XYZ 24x36 landscape drawings",
  "page": {
    "orientation": "landscape",
    "roiSpace": "visual"
  },
  "sheetId": {
    "rois": [
      {
        "x": 0.78,
        "y": 0.05,
        "width": 0.20,
        "height": 0.18
      }
    ],
    "anchorKeywords": ["SHEET", "SHEET NO", "DWG NO"]
  },
  "sheetTitle": {
    "rois": [
      {
        "x": 0.55,
        "y": 0.05,
        "width": 0.40,
        "height": 0.18
      }
    ],
    "maxLength": 100
  },
  "validation": {
    "allowedPrefixes": ["M", "E", "P", "A", "C", "S"]
  }
}
```

## Common ROI Misconfiguration Warnings

### Low Success Rate (<80%)

**Symptoms:**
- Many pages show "ROI empty" or "ROI no pattern match" warnings
- Success rate below 80% in detect output

**How to Fix:**
1. **Check ROI coordinates**: Use `detect` with `--verbose` to see what's being found
2. **Try multiple ROIs**: Add fallback ROI in the `rois` array
3. **Adjust anchor keywords**: Add variations like "SHT", "DWG", "DRAWING NO"
4. **Check page orientation**: Ensure `orientation` matches your PDFs

### No IDs Detected

**Symptoms:**
- All pages show "ROI empty" or "ROI no pattern match"

**How to Fix:**
1. **Verify ROI covers title block**: Coordinates might be off
2. **Check if text is extractable**: Open PDF and try to select text
3. **Try legacy detection**: Remove `--layout` to see if auto-detection works
4. **Check regex pattern**: Custom pattern might be too restrictive

### False Positives

**Symptoms:**
- Detected IDs don't match what you see in the PDF
- Multiple incorrect matches per page

**How to Fix:**
1. **Add validation**: Use `allowedPrefixes` to filter by discipline
2. **Tighten ROI**: Make the region smaller to exclude other text
3. **Adjust anchor keywords**: More specific keywords reduce false matches

## File Organization Best Practices

### Recommended Structure

```
project-root/
├── layouts/                    # Layout profiles (version controlled)
│   ├── layout-template.json
│   └── my-project-layout.json
│
├── conset-output/              # All outputs (gitignored)
│   ├── inventories/           # Master inventory index
│   ├── previews/              # Detection previews
│   ├── reports/               # Merge reports
│   └── merged/                # Final PDFs
│
└── source-pdfs/               # Source files
    ├── original/
    └── addenda/
```

### Benefits

- **Clean separation**: Outputs don't clutter source directories
- **Easy cleanup**: Delete `conset-output/` to remove all generated files
- **Version control**: Keep layouts, ignore outputs
- **Multi-project**: Each project has its own structure

For complete output file organization details, see [OUTPUT_STRUCTURE.md](OUTPUT_STRUCTURE.md) in the docs directory.
