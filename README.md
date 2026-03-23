# Conset PDF

**Deterministic-first, compiler-model system for extracting and reconstructing structured content from AEC PDFs.**

**Version:** 4.2.1 | **Status:** 🔵 Phase 0 (Closeout Complete) | **License:** Apache-2.0

---

## Project Overview

Conset PDF is a production-grade PDF processing engine designed specifically for Architecture, Engineering, and Construction (AEC) document workflows. It extracts structured content from specifications, drawings, and submittals with deterministic outputs, complete audit trails, and the capability to regenerate documents with surgical precision.

**Core capabilities:**
- **Deterministic extraction:** Same input + same profile + same engine version = identical output (bit-for-bit)
- **Structured parsing:** PDF → LayoutIR (intermediate representation) → Medium-specific AST → Regenerated PDF
- **Auditability:** Complete JSONL event logs with confidence scores, decision rationale, and visual overlays
- **Chrome preservation:** Headers, footers, and metadata preservation for professional regenerated documents
- **Medium-specific processing:** Separate, specialized processors for specifications, drawings, and submittals
- **Partial success handling:** Extract what is certain, flag ambiguities—never silently fail or discard work

**Core promise:** AEC users get one-button workflows with same results every time, outputs that can be trusted, and provable audit trails showing exactly what happened.

---

## Problem Statement

AEC document management today is fragmented and unreliable:

- **No determinism:** PDF extraction tools produce different results on identical inputs
- **Silent failures:** Ambiguous parsing succeeds with guesses; users discover errors in production
- **No auditability:** When extraction fails, users have no evidence of what went wrong
- **Medium conflation:** Tools treat specs like drawings, drawings like submittals—all wrong
- **Lost metadata:** Regenerated documents lose project context, branding, professional formatting
- **Irreversible extraction:** Once content is extracted, regenerating it with edits requires manual re-typesetting

Conset solves these by treating PDF extraction as a **deterministic compiler problem**, not a machine learning problem.

---

## Architecture

### High-Level Design

Conset follows a **compiler model**: structured stages with clear inputs/outputs, each independently testable and explainable.

```
PDF Input
    ↓
┌───────────────────────────────────┐
│  Lexer: Layout Extraction         │ → LayoutTranscript (raw geometry + text)
└───────────────────────────────────┘
    ↓
┌───────────────────────────────────┐
│  Parser: Semantic Analysis        │ → DocumentAST (hierarchical structure)
├─ Furniture Detection              │
├─ Content Classification           │
├─ Footer/Header Oracle             │
└───────────────────────────────────┘
    ↓
┌───────────────────────────────────┐
│  Optimizer: Editing & Validation  │ → EditableDocModel (structure with edits applied)
└───────────────────────────────────┘
    ↓
┌───────────────────────────────────┐
│  Code Generator: Rendering        │ → Output PDF + Audit Bundle
└───────────────────────────────────┘
    ↓
Audit Trail (JSONL events + visual overlays)
```

### Workspace Structure

```
apps/
├── backend-cli/         # Workflow entrypoint and contracts↔engine translation layer
└── desktop-gui/         # Tauri desktop surface (frontend wiring deferred)

crates/
├── contracts/           # Canonical request/response + audit schemas
├── workflows/           # Workflow orchestration contracts and stubs
├── standards-data/      # Standards dataset scaffolding
├── engine/              # Deterministic pipeline wrappers and stage orchestration
├── ir/                  # Layout IR and validation semantics
├── audit/               # Audit event bundle models and persistence
└── pdf-extraction/      # PDF loading/text extraction implementation details

tests/
├── corpus/              # Torture corpus fixtures
└── integration/         # Cross-boundary integration scaffolds
```

### Key Design Principles

1. **Determinism is Sacred:** No randomness. Every decision is logged and reproducible.
2. **Correctness Over Speed:** Readable, proven code beats clever optimizations.
3. **Explicit Failure:** Ambiguity produces warnings with evidence, never silent guesses.
4. **Medium-Specificity:** Separate processors for specs, drawings, submittals—no over-generalization.
5. **Audit Trail First:** Every decision logged with confidence, basis, and timestamp.
6. **Chrome Preservation:** Metadata (project ID, section numbers, dates) extracted and reapplied.
7. **Partial Success:** Extract what is certain; flag 20% that needs review instead of discarding 100%.

See [docs/v4/ARCHITECTURE_v4_2.md](docs/v4/ARCHITECTURE_v4_2.md) for complete system design.

---

## Getting Started

### Prerequisites

- **Rust 1.82+** (install via [rustup](https://rustup.rs/))
- **PDFium** (bundled; auto-downloaded on first build)
- **Cargo** (included with Rust)

### Build

```bash
# Clone repository
git clone https://github.com/your-org/conset-pdf.git
cd conset-pdf

# Build all crates (workspace)
cargo build --workspace

# Build with optimizations
cargo build --workspace --release
```

### Run Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific crate tests
cargo test -p conset-pdf-ir
cargo test -p conset-pdf-engine

# Run integration tests
cargo test --test '*' --workspace
```

### Quick Start

```bash
# Display CLI help
cargo run --bin backend-cli -- --help

# Extract PDF (example)
cargo run --bin backend-cli -- extract --input sample.pdf --output output/

# Run engine tests
cargo test -p conset-pdf-engine --test end_to_end_test
```

See [docs/SETUP.md](docs/SETUP.md) for detailed environment configuration and troubleshooting.

---

## Development Workflow

### Guiding Philosophy

Development follows **test-driven design (TDD) with micro-tasks**: small, complete units of work that can be implemented, tested, and verified in a single session.

### Branch Strategy

```
main (stable, always builds)
  ├── feature/<feature-name>   (new features, branches from main)
  ├── fix/<issue-name>          (bug fixes, branches from main)
  └── docs/<topic>              (documentation only)
```

**Rules:**
- All branches must pass CI before merge
- No force-pushes to main
- Branches deleted after merge

### Commit Message Format

```
<type>: <subject>

<body (optional)>

Fixes: #<issue-number> (optional)
```

**Types:** `feat`, `fix`, `test`, `docs`, `refactor`, `perf`, `ci`

**Example:**
```
feat: implement footer extraction for specifications

- Added FooterExtractor struct with deterministic parsing
- Handles multi-line footers with project ID, date, section number
- Includes confidence scoring for ambiguous cases
- 95+ test cases covering AEC footer formats

Fixes: #42
```

### Micro-Task Model

Every feature/fix is broken into **micro-tasks**: focused work units (1–4 hours each) that:
- Have a clear, testable acceptance criterion
- Can be reviewed and merged independently
- Produce a working, non-breaking intermediate state
- Include unit + integration tests

**Example micro-task breakdown (Footer Extraction):**
1. ✅ Define `FooterExtractor` struct and traits
2. ✅ Implement basic footer detection and regex matching
3. ✅ Add confidence scoring for ambiguous formats
4. ✅ Integrate with LayoutIR pipeline
5. ✅ Add 50+ test cases (normal, edge cases, malformed)
6. ✅ Document in audit trail
7. ✅ Merge to main

### Testing Requirements

**Standard:** Every function has a test. Every public API is covered. Every phase has integration tests.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_extraction_normal_case() {
        // Arrange
        let footer = "Section 23 82 16 – Plumbing – Page 3 of 50 – January 15, 2026";
        
        // Act
        let result = extract_footer(footer);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().section_id, "23 82 16");
    }
}
```

**Test pyramid:**
- Unit tests (75%): Individual functions, logic branches
- Integration tests (20%): Component interaction, fixture corpus
- E2E tests (5%): Full workflows on torture corpus

**Golden files:** Regression tests use snapshot files. Regenerate with `cargo test -- --nocapture`.

See [docs/v4/DEV_STANDARDS_v4_2.md](docs/v4/DEV_STANDARDS_v4_2.md) for full testing standards.

### Code Review Checklist

Before opening a PR:

- [ ] Tests pass locally (`cargo test --workspace`)
- [ ] No clippy warnings (`cargo clippy --workspace`)
- [ ] Code follows naming conventions (snake_case functions, PascalCase types)
- [ ] Error handling is explicit (Result/Option, no unwrap)
- [ ] Determinism preserved (no randomness, no timing dependencies)
- [ ] Confidence scores assigned (if applicable)
- [ ] Audit events logged (if decision-making)
- [ ] Commit messages follow format
- [ ] Documentation updated (if API change)

### AI Coding Agent Workflow

When working with AI agents (e.g., Claude):

1. **Provide context:** Link relevant standards, architecture docs, existing implementations
2. **Specify acceptance criteria:** What success looks like (tests, behavior, outputs)
3. **Define constraints:** Determinism requirements, error handling, audit logging
4. **Review suggestions:** Don't blindly accept generated code—verify it follows standards
5. **Validate tests:** Run generated tests locally before commit

---

## Documentation

Comprehensive documentation lives in `docs/`:

| Document | Purpose |
|----------|---------|
| [DOCUMENTATION_INDEX.md](docs/DOCUMENTATION_INDEX.md) | Canonical documentation authority and navigation entrypoint |
| [MASTER_PLAN_v4.md](docs/v4/MASTER_PLAN_v4.md) | North Star vision, non-negotiables, roadmap, implementation phases |
| [ARCHITECTURE_v4_2.md](docs/v4/ARCHITECTURE_v4_2.md) | System design, crate structure, compiler pipeline, type system |
| [DEV_STANDARDS_v4_2.md](docs/v4/DEV_STANDARDS_v4_2.md) | Coding standards, testing requirements, Git workflow, code review |
| [AEC_STANDARDS_v4_2.md](docs/v4/AEC_STANDARDS_v4_2.md) | AEC domain knowledge, specs/drawings/submittals, UDS classification |
| [TRANSCRIPT_ARCHITECTURE_v4_2.md](docs/TRANSCRIPT_ARCHITECTURE_v4_2.md) | LayoutTranscript format, geometry encoding, invariants |
| [SETUP.md](docs/SETUP.md) | Environment setup, dependency installation, troubleshooting |
| [PHASE_0_IMPLEMENTATION_PLAN_v4_2_1.md](docs/dev/PHASE_0_IMPLEMENTATION_PLAN_v4_2_1.md) | Detailed Phase 0 task breakdown |

---

## Testing

### Test Organization

```
crates/
├── engine/tests/end_to_end_test.rs       # Full pipeline on fixtures
├── ir/tests/
│   ├── bbox_test.rs                      # Geometry validation
│   ├── normalization_test.rs             # Coordinate normalization
│   ├── layout_test.rs                    # Layout tree invariants
│   └── ...
└── pdf-extraction/tests/
    ├── page_count_test.rs                # PDF parsing
    └── text_extraction_test.rs           # Raw text extraction

tests/
├── corpus/                               # Test PDFs
│   ├── tier1/                            # Simple, well-formed (< 20 pages)
│   ├── tier2/                            # Moderate complexity (20–100 pages)
│   ├── tier3/                            # Complex edge cases (100+ pages, unusual layouts)
│   ├── presort/                          # Pre-screening for new docs
│   └── holdout/                          # Regression suite (locked, never modified)
```

### Torture Corpus

The `tests/corpus/` directory contains a graduated corpus of real AEC PDFs for regression testing:

- **Tier 1:** Well-formed specs and drawings; simple layouts
- **Tier 2:** Complex layouts, mixed formats, edge cases
- **Tier 3:** Hostile inputs, rare formats, boundary conditions
- **Holdout:** Locked test set for nightly regression (bit-identical output required)

Run the corpus:
```bash
cargo test --test end_to_end_test -- --nocapture

# Or specific tier
cargo test --test end_to_end_test tier1
```

### Golden Files

Regression tests use snapshot files. When behavior changes intentionally:

```bash
# Regenerate snapshots (review changes carefully)
cargo test --workspace -- --nocapture

# Accept new snapshots
# (snapshot files are in tests/golden/)
```

Snapshots are committed to version control and reviewed as part of PR process.

---

## Contributing

We welcome contributions that maintain Conset's core values: determinism, correctness, and auditability.

### Contribution Guidelines

1. **Start with an issue:** Discuss proposed changes before coding
2. **Follow standards:** Adhere to [DEV_STANDARDS_v4_2.md](docs/v4/DEV_STANDARDS_v4_2.md)
3. **Test thoroughly:** Unit + integration tests required; aim for 80%+ coverage
4. **Write clear commits:** Use format specified above
5. **Document APIs:** Public methods need doc comments with examples
6. **Log audit events:** If decision-making, emit audit events with confidence

### Pull Request Process

1. **Create branch:** `feature/<name>` or `fix/<name>`
2. **Implement + test:** Work incrementally, commit frequently
3. **Open PR:** Link related issues, summarize changes
4. **Address review:** Respond to feedback, update code
5. **Merge:** Squash-and-merge to main; delete branch

**PR template:**
```markdown
## Description
[What does this change do?]

## Type
- [ ] Feature
- [ ] Bug fix
- [ ] Documentation
- [ ] Refactor

## Testing
- [ ] Unit tests added
- [ ] Integration tests added
- [ ] Manual testing completed

## Checklist
- [ ] Follows DEV_STANDARDS_v4_2
- [ ] Determinism preserved
- [ ] Audit logging included
- [ ] Tests pass locally
- [ ] No clippy warnings

## Related Issues
Fixes #<issue-number>
```

### Reporting Issues

When reporting bugs, include:
- PDF sample (if possible; sanitize sensitive data)
- Exact command run
- Expected vs. actual output
- Audit bundle (from `audit/` directory)
- Rust version (`rustc --version`)

---

## License

Conset PDF is licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) file for details.

**Licensing Policy:**
- Engine code: Apache-2.0 (permissive)
- Dependencies: No GPL/AGPL in core dependency graph
- Compliance: Annual license audit required
- PDFium: Apache-2.0 compatible

---

## Contact & Support

- **Issues:** [GitHub Issues](https://github.com/your-org/conset-pdf/issues)
- **Discussions:** [GitHub Discussions](https://github.com/your-org/conset-pdf/discussions)
- **Documentation:** See `docs/` directory
- **Owner:** HLLMR LLC

---

**Last Updated:** March 23, 2026 | **Maintained By:** Development Team