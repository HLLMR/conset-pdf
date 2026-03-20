## Plan: Disk-Based Streaming PDF Merge via Pikepdf

**TL;DR:** Refactor the merge workflow to use disk-based streaming operations via a Python sidecar (pikepdf), eliminating large in-memory buffers and enabling robust handling of large PDFs. The disk-based method will be used globally for all files, with no threshold or decision gating. Temp file output handling will be implemented to allow safe overwriting of input files if desired.

---

### Phase 1: Merge Plan Generation (Node.js)
- Analyze original and addendum PDFs to determine merge actions (replace, insert, append).
- Generate a detailed merge plan: list of source PDFs, page indices, and intended order.
- Serialize the merge plan to a JSON file (e.g., `merge-plan.json`).
- **Checkpoint:** Merge plan JSON exists and accurately reflects intended output.

**Relevant files:**
- conset-pdf/packages/core/src/core/mergeAddenda.ts — merge orchestration
- conset-pdf/packages/core/src/core/planner.ts — merge plan generation

---

### Phase 2: Disk-Based Merge Execution (Python Sidecar)
- Python script reads the merge plan JSON.
- Uses pikepdf to open each source PDF sequentially.
- Streams/copies required pages directly to the output PDF, writing to disk (not memory).
- Closes each PDF after processing its pages.
- Handles errors gracefully (e.g., missing files, memory issues).
- Always writes to a temp output file, then atomically renames or overwrites the intended output file (including input file if user requests overwrite).
- **Checkpoint:** Output PDF is created on disk, matching the merge plan.

**Relevant files:**
- New Python script (e.g., `merge_pdf_sidecar.py`) in conset-pdf/packages/core/src/bookmarks/sidecar/ or similar
- Merge plan JSON file

---

### Phase 3: Node.js Integration & Error Handling
- Node.js invokes the Python sidecar via child_process, passing merge plan and output path.
- Monitors process for errors, timeouts, and completion.
- Cleans up temp files and reports errors to user/UI.
- On success, continues workflow (e.g., report generation, bookmarks).
- **Checkpoint:** Merge operation completes successfully, errors are surfaced and handled.

**Relevant files:**
- conset-pdf/packages/core/src/utils/pikepdfWriter.ts — refactor to call new Python script
- conset-pdf/packages/core/src/core/applyPlan.ts — update merge logic

---

### Phase 4: Verification & Reporting
- Validate output PDF (page count, order, bookmarks if applicable).
- Generate merge report (JSON) for downstream workflows.
- Optionally, run automated tests for large file handling.
- **Checkpoint:** Output PDF and report are verified, ready for downstream use.

**Relevant files:**
- conset-pdf/packages/core/src/core/report.ts — merge report generation
- Test files in conset-pdf/packages/core/tests/

---

## Decisions
- Disk-based streaming via pikepdf is the primary merge pathway for all files (no threshold or gating).
- Merge plan is serialized for cross-language handoff.
- Python sidecar is responsible for all disk-based PDF manipulation.
- Node.js handles orchestration, error handling, and reporting.
- Temp file output handling ensures safe overwriting of input/output files as requested by the user.

## Further Considerations
1. Batch processing in Python sidecar for extremely large sets (optional).
2. User feedback for memory/disk errors and recovery options.
