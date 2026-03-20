# Phase 09 — Deprecation Gates and Behavioral Flags

**Purpose**: Capture all deprecation controls in the prototype, their defaults, their failure messaging, and what they mean for Rust feature-toggle and compatibility shim design.

---

## 1. Overview of Deprecation Architecture

The prototype uses two mechanisms for managing deprecated behavior:

1. **Feature flags** (`packages/core/src/config/featureFlags.ts`) — runtime toggles for experimental or abandoned features
2. **Deprecation logging utilities** (`packages/core/src/utils/deprecation.ts`) — structured console warnings emitted when deprecated code paths are invoked

Both are exported from `packages/core/src/index.ts` as part of the public API surface.

---

## 2. Feature Flag Registry

### 2.1 `ENABLE_LEGACY_LOCATOR`

| Property | Value |
|----------|-------|
| **Default** | `false` |
| **Environment variable** | `ENABLE_LEGACY_LOCATOR=true` (or `=1`) |
| **Scope** | Core library, `CompositeLocator` |
| **Status** | Permanently gated; not a compatibility shim — legacy detection was superseded |

**Behavior when OFF (default)**:
- `CompositeLocator` blocks legacy fallback if ROI detection fails
- Returns an error result with the message: `"Legacy title block detection is disabled. Enable it with ENABLE_LEGACY_LOCATOR=true or fix your ROI profile."`
- Emits deprecation warning via `logLegacyLocatorUsage()` pointing to ROI profile documentation

**Behavior when ON** (`ENABLE_LEGACY_LOCATOR=true`):
- Legacy fallback is allowed, same behavior as pre-deprecation
- Deprecation warning is still emitted on every use (logged, not thrown)
- `LegacyTitleblockLocator` is invoked; marked in its JSDoc as "Will be removed in v4.0.0"

**Runtime API** (exported from `@conset-pdf/core`):
```typescript
import { isLegacyLocatorEnabled, setFeatureFlag, getFeatureFlag, resetFeatureFlags } from '@conset-pdf/core';

// Check current state
isLegacyLocatorEnabled(); // → false by default

// Enable programmatically (for tests or compatibility)
setFeatureFlag('ENABLE_LEGACY_LOCATOR', true);

// Reset to defaults
resetFeatureFlags();
```

**Test implications**: The smoke test suite (`tests/smoke/core-behaviors.test.ts`) explicitly enables the flag to test legacy fallback path. Any test relying on legacy fallback must call `setFeatureFlag('ENABLE_LEGACY_LOCATOR', true)` before the test and reset after.

**Failure message** (when flag is OFF and ROI fails):
```
Legacy title block detection is disabled. Enable it with ENABLE_LEGACY_LOCATOR=true or fix your ROI profile.
```

---

## 3. Deprecation Logging Utilities

### 3.1 `logDeprecation(feature, message, alternative?, moreInfo?)`
Generic deprecation logger. Emits a `console.warn` with structured context:
- Feature name
- Deprecation message
- Alternative (if provided)
- Link/doc reference (if provided)

### 3.2 `logLegacyLocatorUsage(context?)`
Specific warning for legacy locator use. Emitted at:
1. When flag is OFF and legacy fallback is blocked
2. When flag is ON and legacy fallback proceeds

### 3.3 `logPdfAstDeprecation()`
Placeholder for PDF AST system deprecation. Declared but not yet called from any active code. Reserved for future cleanup of any remaining PDF AST references.

---

## 4. Permanently Removed vs. Gated Behavior

| Feature | State | Implementation |
|---------|-------|---------------|
| **Legacy Titleblock Locator** | Gated OFF by default | `featureFlags.ENABLE_LEGACY_LOCATOR = false`; code still on disk; will be removed in v4.0.0 |
| **PDF AST system** | Removed from active workflows; files may remain | `logPdfAstDeprecation()` placeholder exists; no active callers |
| **`pdfLibBookmarkWriter`** | Retained for development/testing only | Not invoked by production code paths; `pikepdfBookmarkWriter` is the production path |
| **`utils/bookmarks.ts` legacy generators** | Deprecated but not removed | Marked as deprecated in JSDoc; replaced by `bookmarkWriter.ts` |
| **Assemble Set workflow** | Abandoned — files on disk, no active wiring | `commands/assembleSet.ts` exists but is not registered as an active CLI command in production flow |
| **Specs Patch workflow** | Abandoned — files on disk | `commands/specsPatch.ts` and `workflows/specs-patch/` exist but classified as abandoned |
| **`--auto-layout` / `--save-layout` CLI flags** | Declared in CLI, never implemented | Registered as Commander options in `mergeAddenda.ts` but handler never references `options.autoLayout` or `options.saveLayout` — effectively no-ops |

---

## 5. Deprecation Gate Behavior vs. Hard Removal Contrast

The prototype chose **soft gating** (flag + warning) over **hard removal** for the legacy locator for the following reason:
- Some projects may rely on title-block-auto-detection while migrating to ROI profiles
- The gate allows an escape hatch without committing indefinitely to maintain the code

The `--auto-layout` CLI flag took the opposite (worse) approach: it was added to the option list to reserve the interface but implemented nothing behind it. This is the pattern to avoid in Rust.

---

## 6. `FeatureFlagConfig` Schema

```typescript
interface FeatureFlagConfig {
  ENABLE_LEGACY_LOCATOR: boolean; // Default: false
}
```

Currently only one flag exists. The infrastructure is designed for future expansion but the prototype exited with a single flag. The `featureFlags.ts` module is minimal by design.

---

## 7. Deprecation Warning Format

All deprecation warnings follow this console.warn format (from `deprecation.ts`):
```
[DEPRECATED] <feature>: <message>
  Alternative: <alternative>
  More info: <url-or-doc-ref>
```

This is console-only (not persisted to log files in the CLI). The GUI's structured logger would need integration if log persistence is required.

---

## 8. Rust Implementation Implications

### 8.1 Feature Toggle Design

| Constraint | Rationale |
|-----------|-----------|
| Legacy locator must be **opt-in only** in Rust | The ROI profile system is the sole supported path; legacy detection is a compatibility escape hatch, not a default |
| Feature flags should be env-var driven at minimum | Tests need programmatic override without recompilation; `ENABLE_LEGACY_LOCATOR` pattern is acceptable |
| No implicit silent fallback to deprecated code paths | If a gate is OFF and the gated path is needed, return a clear error (not a silent fallback) |
| Dead flags must not ship | `--auto-layout` must not appear in the Rust CLI unless it is actually implemented |

### 8.2 Compatibility Shims vs. Permanent Removal

| Item | Rust Recommendation |
|------|---------------------|
| Legacy titleblock locator | Do not port. ROI profile system replaces it completely. Remove without shim. |
| pdf-lib bookmark writer | Do not port. QPDF/lopdf replaces it. |
| `utils/bookmarks.ts` legacy generators | Do not port. Superseded by canonical bookmark writer. |
| Specs Patch workflow | Do not port as a standalone workflow. Functionality merged into Extract workflow design. |
| Assemble Set workflow | Do not port. Replaced by Extract Documents output composition. |
| `--auto-layout` flag | Implement properly (Phase 3 V4) or do not expose the flag. Never declare a no-op flag. |

### 8.3 Flag Naming Convention

The prototype uses SCREAMING_SNAKE_CASE env vars for feature flags (`ENABLE_LEGACY_LOCATOR`). Rust's standard env-var naming is the same. Carry this convention forward when feature toggles are needed in the Rust successor.

---

## 9. Known Ambiguity

The prototype `DEPRECATION_CHANGES.md` refers to "Published API Exports" for feature flag functions. This means consumers of `@conset-pdf/core` could programmatically enable the legacy locator at runtime from external code (e.g., a GUI layer). This is intentional for migration compatibility, but has a security-adjacent concern: any caller can re-enable deprecated behavior without a restart.

**Rust successor decision**: Feature flags that gate security-relevant behavior should require restart (env var only) rather than allowing runtime mutation. For the legacy locator, runtime mutation is acceptable since it controls detection accuracy, not security.
