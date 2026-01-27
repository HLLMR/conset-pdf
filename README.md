# Conset PDF

**Version:** 4.2.0  
**Status:** 🔵 In Development (Phase 0)

[![CI](https://github.com/your-org/conset-pdf/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/conset-pdf/actions/workflows/ci.yml)
[![Clippy](https://github.com/your-org/conset-pdf/actions/workflows/clippy.yml/badge.svg)](https://github.com/your-org/conset-pdf/actions/workflows/clippy.yml)
[![Coverage](https://codecov.io/gh/your-org/conset-pdf/branch/master/graph/badge.svg)](https://codecov.io/gh/your-org/conset-pdf)

## Overview

Conset PDF is a deterministic-first PDF processing engine for AEC (Architecture, Engineering, Construction) documents. It extracts, parses, and reconstructs structured content from specs, drawings, and submittals with audit trails and reproducible results.

## Status

Currently in **Phase 0: Project Scaffolding**

- [x] Documentation suite complete
- [ ] Cargo workspace setup
- [ ] PDF extraction library selected
- [ ] LayoutIR implementation
- [ ] CI/CD pipeline

See [docs/master_plan_v4_2.md](docs/master_plan_v4_2.md) for full roadmap.

## Project Structure

- **docs/** - Project documentation (Master Plan, standards, architecture)
- **crates/** - Rust crates (workspace members)
  - **engine/** - Main binary and CLI
  - **ir/** - Layout IR (intermediate representation)
  - **audit/** - Audit framework
  - **pdf-extraction/** - PDF library wrapper
- **tests/** - Integration tests and fixtures
- **tools/** - Development tools

## Quick Start
```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run engine
cargo run --bin conset-pdf -- --help
```

See [docs/SETUP.md](docs/SETUP.md) for full environment setup and PDFium installation.

## Documentation

- [Master Plan V4.2](docs/master_plan_v4_2.md) - Project strategy and phasing
- [DEV_STANDARDS V4.2](docs/DEV_STANDARDS_v4_2.md) - Coding standards
- [ARCHITECTURE V4.2](docs/ARCHITECTURE_v4_2.md) - System design

## Development

This project follows strict determinism and auditability standards. See [DEV_STANDARDS](docs/DEV_STANDARDS_v4_2.md) for:
- Micro-task workflow
- Test-driven development
- Debug logging requirements
- Code review checklist

## License

Apache-2.0

## Author

HLLMR LLC