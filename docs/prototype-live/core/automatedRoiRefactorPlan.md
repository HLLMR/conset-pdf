## Plan: Automated ROI Detection Refactor

TL;DR: Refactor the PDF processing pipeline to enable robust, automated ROI (Region of Interest) detection and profile generation using Python (pikepdf, PyMuPDF), replacing manual layout profile creation and legacy deterministic fallback. This will streamline sheet identification, improve accuracy, and reduce manual intervention.

**Steps**

### Phase 1: Requirements & Analysis
1. Define target ROI types (e.g., title block, sheet number, revision block) and their expected characteristics.
2. Review and document the current layout profile schema ([conset-pdf/layouts/layout-template.json](conset-pdf/layouts/layout-template.json)).
3. Collect representative sample PDFs for algorithm development and testing.

### Phase 2: Engine & Data Extraction
4. Implement or extend Python sidecar scripts to extract page-level text, images, and layout metadata using PyMuPDF.
5. Use pikepdf for structural PDF analysis (object positions, bookmarks, etc.).
6. Normalize extracted data for downstream processing.

### Phase 3: Detection Algorithms
7. Develop deterministic/heuristic algorithms for ROI detection:
   - Text pattern matching (regex for sheet numbers, titles, etc.)
   - Spatial analysis (bounding boxes, coordinates)
   - Visual cues (lines, shapes, logos)
8. Validate detected ROIs against the layout profile schema.

### Phase 4: Profile Generation & Integration
9. Convert detected ROIs into layout profile JSON format.
10. Integrate auto-generated profiles with the existing profile management system (CLI/GUI).
11. Support batch processing for multiple PDFs.
12. Allow manual override/editing of auto-generated profiles in GUI.

### Phase 5: Testing, Feedback & Iteration
13. Test detection on diverse PDF samples; log results and errors.
14. Refine detection heuristics based on test outcomes and user feedback.
15. Document limitations, fallback scenarios, and update user guides.

**Relevant files**
- conset-pdf/layouts/layout-template.json — profile schema reference
- conset-pdf/packages/core/src/ — core PDF processing logic
- conset-pdf/scripts/ — Python sidecar scripts
- conset-pdf-gui/src/profiles-view.js — GUI profile management
- conset-pdf-gui/docs/UI_WORKFLOWS.md — workflow documentation

**Verification**
1. Automated tests: Validate ROI detection accuracy on sample PDFs.
2. Manual review: Confirm auto-generated profiles match expected layouts.
3. GUI integration: Ensure manual editing and override works as intended.
4. Error logging: Check logs for detection failures and edge cases.

**Decisions**
- Automation is prioritized; deterministic fallback remains for edge cases.
- Python (pikepdf, PyMuPDF) is the canonical engine for ROI detection.
- Manual profile editing is retained for flexibility.

**Further Considerations**
1. Consider future support for additional ROI types or custom detection rules.
2. Evaluate performance on large PDFs and optimize as needed.
3. Plan for incremental rollout and user training.
