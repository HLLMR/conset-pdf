# Phase 09 — Documentation Drift Matrix

**Purpose**: Authoritative cross-reference of what each shipping document claims vs. what the code and tests actually demonstrate. A Rust developer must use code truth here, not any single markdown source.

**Sources compared**:
- `README.md` (last verified 2026-03-01)
- `ROADMAP.md` (last updated 2026-01-17 per footer, "Last updated: 2026-03-01" per header — inconsistent metadata)
- `docs/WORKFLOWS.md` (last verified 2026-03-01)
- `docs/CLI.md` (last verified 2026-03-01)
- Code: `packages/cli/src/commands/`, `packages/core/src/workflows/`, `conset-pdf-gui/src/`

---

## Matrix Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Claim confirmed by code/tests |
| ⚠️ | Partially confirmed or ambiguous |
| ❌ | Claim contradicted by code/tests |
| 🔄 | Internal doc contradiction (doc disagrees with itself) |
| ☠️ | Dead code / abandoned but not removed |

---

## 1. Workflow Implementation Status

### 1.1 Update Documents (Merge) Workflow

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "✅ Fully Implemented (Engine + CLI + GUI)" | README.md, ROADMAP summary table | `workflows/merge/`, `commands/mergeAddenda.ts`, `conset-pdf-gui/src/merge-wizard.js` — all present and active | ✅ CONFIRMED |
| "4-step Update Documents wizard complete" | ROADMAP GUI section, GUI DEVELOPMENT.md | Merge wizard 4-step flow verified in `merge-wizard.js` and UI_WORKFLOWS.md | ✅ CONFIRMED |
| "Narrative validation UI integration complete" | ROADMAP GUI Complete section | UI_WORKFLOWS.md Step 3 describes narrative validation UI; DEVELOPMENT.md notes "narrative PDF path stored in wizard state but not parsed/used" | ⚠️ PARTIAL — narrative path is stored but core parsing not wired to GUI in all paths |
| "analyze → applyCorrections → execute pattern" | WORKFLOWS.md, README.md | Confirmed in `workflows/merge/`, `MergeExecuteInput.analyzed` not used in Phase 1 | ⚠️ PARTIAL — phase 1 doesn't use cached plan from analyze |

### 1.2 Extract Documents (Split) Workflow

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "✅ Fully Complete (Engine + CLI)" | ROADMAP narrative (implied by Complete table) | `workflows/` split engine present; `commands/splitSet.ts` present | ✅ CONFIRMED |
| "GUI: ✅ Complete" | ROADMAP **summary table** row | GUI section of ROADMAP says "⚠️ Placeholder UI Exists / Blocked By: Core workflow engines not implemented" for split wizard | ❌ **DRIFT — TABLE IS WRONG** — GUI wizard is placeholder only |
| "Split/Assemble workflows - CLI commands exist, workflow engine not yet implemented" | README.md ⚠️ Partially Implemented section | Engine IS implemented for split; assemble was abandoned | 🔄 **STALE README ENTRY** — correct for assemble, wrong for split |
| "Blocked By: Core workflow engines not implemented" | ROADMAP GUI section | Split engine is implemented; this note is stale | ❌ **STALE** — the blocker was resolved, note not removed |

**Code Truth**: The split workflow engine and CLI are complete. The GUI wizard is a placeholder (`split-drawings-wizard.js`, `placeholder-wizard.js`). ROADMAP summary table erroneously marks GUI as ✅. The "Blocked By" note in the GUI section is stale.

### 1.3 Fix Bookmarks Workflow

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "✅ 100% Complete (Engine + CLI)" | ROADMAP narrative section | `workflows/bookmarks/`, `commands/fixBookmarks.ts` present and active | ✅ CONFIRMED |
| "GUI: ✅ Complete" | ROADMAP **summary table** | ROADMAP GUI section: "Fix Bookmarks wizard: ⚠️ Placeholder UI Exists / Blocked By" | ❌ **DRIFT — TABLE IS WRONG** — GUI wizard is placeholder only |
| "Footer-First Section Anchoring complete" | README.md, ROADMAP | Confirmed active in `bookmarks/` module | ✅ CONFIRMED |

**Code Truth**: Fix Bookmarks engine and CLI complete. GUI wizard is a placeholder. Summary table is wrong.

### 1.4 Specs Patch Workflow

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "✅ 100% Complete (Engine + CLI)" | ROADMAP ✅ Fully Complete section | `workflows/specs-patch/`, `commands/specsPatch.ts` files exist on disk | 🔄 **INTERNAL CONTRADICTION** |
| "❌ Abandoned — Superseded By: Extract Documents workflow" | ROADMAP ❌ Abandoned section | Same files exist; CLI.md marks as abandoned | 🔄 **DUPLICATE ENTRY** — workflow appears as both Complete and Abandoned |
| Not mentioned as active | CLI.md commands list | CLI.md lists it under "Abandoned/Superseded" | ⚠️ Consistent with abandonment but archive not cleaned |
| "specs-patch, assemble (abandoned)" | README.md workflow types line | Consistent with abandonment | ✅ CONSISTENT |

**Code Truth**: The specs-patch files exist on disk (not deleted) but are classified as abandoned. The ROADMAP incorrectly lists it as a completed workflow in the ✅ section AND as abandoned in the ❌ section. Treat as **abandoned**. The files are dead code preserved for reference.

### 1.5 Assemble Set Workflow

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "❌ Abandoned — Replaced by modular composition in Extract Documents" | ROADMAP | `commands/assembleSet.ts` exists on disk | ☠️ DEAD CODE — file present but abandoned |
| Listed under "Abandoned/Superseded" | CLI.md | Consistent with ROADMAP | ✅ CONSISTENT |

---

## 2. CLI Command Status

| Command | CLI.md Status | ROADMAP Status | Code File | Verdict |
|---------|-------------|---------------|-----------|---------|
| `merge-addenda` | Active | ✅ Complete | `mergeAddenda.ts` | ✅ Active |
| `split-set` | Active | ✅ Complete | `splitSet.ts` | ✅ Active |
| `fix-bookmarks` | Active | ✅ Complete | `fixBookmarks.ts` | ✅ Active |
| `detect` | Active | Active | `detect.ts` | ✅ Active |
| `specs-patch` | Abandoned/Superseded | ❌ Abandoned | `specsPatch.ts` | ☠️ Dead code on disk |
| `assemble-set` | Abandoned/Superseded | ❌ Abandoned | `assembleSet.ts` | ☠️ Dead code on disk |
| `debug-walkthrough` | Not documented | Not documented | `debugWalkthrough.ts` | ⚠️ Developer-only, not public |
| `specs-inventory` | Not documented | Not documented | `specsInventory.ts` | ⚠️ Undocumented command on disk |

**New Finding**: `debugWalkthrough.ts` and `specsInventory.ts` exist in `commands/` but are not mentioned in CLI.md, README.md, or ROADMAP.md. These are either developer-only tools or abandoned commands. A Rust implementation should verify before discarding.

---

## 3. CLI Flag Accuracy

### 3.1 `--auto-layout` and `--save-layout` Flags

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "`--auto-layout`: Auto-detect layout and suggest profile" | CLI.md, README.md | Flag declared in `mergeAddenda.ts` (line 52) but `options.autoLayout` is **never referenced** in the action handler | ❌ **DEAD FLAG** — declared but not implemented |
| "`--save-layout <path>`: Save auto-detected layout to file" | CLI.md | Flag declared but `options.saveLayout` never referenced in handler | ❌ **DEAD FLAG** — declared but not implemented |

**Code Truth**: Both `--auto-layout` and `--save-layout` appear in CLI documentation and are registered as CLI options but have no backend implementation. The automatedRoiRefactorPlan.md confirms auto-detection was designed but never built. These flags are no-ops.

**Rust implication**: Do not implement auto-layout as V3 behavior — it was never delivered. Treat as a V4 Phase 3 feature.

### 3.2 `--bookmark` Flag (merge-addenda)

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "Regenerate bookmarks from detected sheet numbers and titles" | CLI.md, README.md | `mergeAddenda.ts` passes `options.bookmark` → `regenerateBookmarks` in execute input | ✅ CONFIRMED — wired through |

### 3.3 Exit Codes

| Claim | Source | Verified |
|-------|--------|---------|
| 0=Success, 2=Invalid args, 3=Strict mode, 4=File I/O | CLI.md | Matches `process.exit()` calls in handler | ✅ CONFIRMED |

---

## 4. Workflow Engine Behavior

### 4.1 Corrections Overlay in execute()

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "analyze → applyCorrections → execute" is full three-phase pattern | WORKFLOWS.md, README.md | `applyCorrections()` re-runs `analyze()` then overlays corrections. `execute()` accepts `analyzed` and `corrections` but Phase 1 **ignores both** (re-runs analysis internally) | ⚠️ PARTIAL — the pattern is architected but execute re-derives analysis in Phase 1 |

This is documented in the code comments and confirmed by DEVELOPMENT.md: "Multi-lane merge in core (GUI supports 'both' scope but core executes lanes separately)". The workflow corrections overlay is loaded on `execute()` but not applied in the first merge driver implementation.

### 4.2 Multi-Addendum Support

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "Supports multiple addenda in chronological order (`--addenda <paths...>`)" | CLI.md | CLI accepts array; analyze/execute do process multiple addenda | ✅ CONFIRMED |
| "Single addendum per wizard run" | UI_WORKFLOWS.md | Wizard forces single addendum per run | ✅ CONFIRMED (GUI is single-addendum by design) |

---

## 5. Standards Module

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "UDS-style discipline identification complete" | README.md, ROADMAP | `standards/normalizeDrawingsDiscipline.ts` present | ✅ CONFIRMED |
| "CSI MasterFormat classification complete" | README.md, ROADMAP | `standards/normalizeSpecsMasterformat.ts` present | ✅ CONFIRMED |
| Integrated into merge workflow | WORKFLOWS.md | Discipline/division fields present in `InventoryResult` rows | ✅ CONFIRMED |

---

## 6. Transcript / V3 Architecture

| Claim | Source | Code Truth | Verdict |
|-------|--------|-----------|---------|
| "PyMuPDF primary, PDF.js fallback" | README.md | `transcript/` factory and extractor confirm backend selection | ✅ CONFIRMED |
| "95-99% bbox accuracy with PyMuPDF" | README.md | Per Phase 2/6 analysis; no CI benchmark enforces this claim | ⚠️ ASPIRATIONAL — no automated assertion |
| "PDF.js demoted from primary to fallback" | ADR-002 | Confirmed in factory logic | ✅ CONFIRMED |
| "Full determinism" | README.md, multiple | `canonicalize.ts` implements stable sort, rotation normalization, deterministic hashes | ✅ CONFIRMED |

---

## 7. GUI Module Inventory vs ROADMAP Claims

| Module | ROADMAP GUI Claim | Actual State |
|--------|-------------------|-------------|
| Update Documents (Merge) wizard | ✅ Complete | Complete 4-step wizard |
| Profile management with ROI selection | ✅ Complete | Implemented in `profiles-view.js` |
| Extract Documents (Split) wizard | ⚠️ Placeholder / ❌ Table says ✅ | **Placeholder only** (`split-drawings-wizard.js`, `placeholder-wizard.js`) |
| Fix Bookmarks wizard | ⚠️ Placeholder / ❌ Table says ✅ | **Placeholder only** |
| Report Viewer | Not in ROADMAP | **Never implemented** — no UI shell exists |
| Specs GUI wizard | Not in ROADMAP GUI section | Not implemented |
| Submittal workflow | Not in ROADMAP GUI section | Not implemented |

---

## 8. Migration Gate Decision

**For Rust planning**: The following table represents the authoritative behavioral truth, overriding any single doc source:

| Feature | Is Implemented? | Notes |
|---------|----------------|-------|
| Merge workflow (core + CLI) | YES | Use as reference |
| Merge wizard (GUI) | YES | 4-step flow, single addendum |
| Split workflow (core + CLI) | YES | Use as reference |
| Split wizard (GUI) | NO | Placeholder only |
| Fix Bookmarks workflow (core + CLI) | YES | Use as reference |
| Fix Bookmarks wizard (GUI) | NO | Placeholder only |
| Specs Patch workflow | NO (abandoned) | Files on disk are dead code |
| Assemble Set workflow | NO (abandoned) | Files on disk are dead code |
| Auto-layout detection | NO | CLI flag is a no-op; design only |
| Narrative parsing (core) | YES | Algorithmic; LLM integration not done |
| Narrative validation UI (GUI) | PARTIAL | Path stored, full integration incomplete |
| Standards module | YES | UDS + MasterFormat |
| ROI detection | YES | With profiles |
| Legacy titleblock detection | GATED (off by default) | Behind `ENABLE_LEGACY_LOCATOR` flag |
| Transcript (V3) extraction | YES | PyMuPDF primary |

---

## 9. Open Drift Items for doc-fixing (Not Blocking for Rust)

1. ROADMAP summary table should show ⚠️ for Extract Documents GUI and Fix Bookmarks GUI (not ✅)
2. ROADMAP duplicate Specs Patch entry should be collapsed to just ❌ Abandoned
3. README.md "Partially Implemented" entry for Split/Assemble should be removed or updated
4. CLI.md should note that `--auto-layout` and `--save-layout` are not implemented
5. `debugWalkthrough.ts` and `specsInventory.ts` commands should be documented or explicitly marked as developer-only
6. ROADMAP footer/header date inconsistency should be resolved ("Last updated: 2026-01-17" vs header "2026-03-01")
