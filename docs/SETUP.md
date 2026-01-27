# Conset PDF Development Setup

## Development Prerequisites

### Required Software

1. **Rust Toolchain**
   - Version: 1.82 or later
   - Install: https://rustup.rs/
   - Verify: `rustc --version`

2. **PDFium Library**
   - Platform-specific binaries required for PDF processing
   - Download from: https://github.com/bblanchon/pdfium-binaries/releases

   **Windows:**
   ```powershell
   # Download latest chromium build
   Invoke-WebRequest -Uri "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6666/pdfium-windows-x64.tgz" -OutFile pdfium.tgz
   tar -xzf pdfium.tgz

   # Option A: Copy to project root
   Copy-Item bin/pdfium.dll ./pdfium.dll

   # Option B: Set environment variable
   $env:PDFIUM_LIB_PATH = "C:\path\to\pdfium\bin"
   ```

   **macOS:**
   ```bash
   # Download latest chromium build
   curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6666/pdfium-darwin-arm64.tgz" -o pdfium.tgz
   tar -xzf pdfium.tgz

   # Copy to project root
   cp lib/libpdfium.dylib ./

   # Or set environment variable
   export PDFIUM_LIB_PATH=/path/to/pdfium/lib
   ```

   **Linux:**
   ```bash
   curl -L "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6666/pdfium-linux-x64.tgz" -o pdfium.tgz
   tar -xzf pdfium.tgz
   cp lib/libpdfium.so ./
   export PDFIUM_LIB_PATH=/path/to/pdfium/lib
   ```

3. **Poppler Utils** (Optional, for classify-pdf tool)
   - Provides `pdfinfo` command for metadata extraction
   - Windows: Download from https://blog.alivate.com.au/poppler-windows/
   - macOS: `brew install poppler`
   - Linux: `apt-get install poppler-utils`
   - Note: classify-pdf works without this but has reduced accuracy

### Verification

After setup, verify everything works:
```bash
# Build all crates
cargo build

# Run tests
cargo test

# Test PDF classification tool
cargo run --bin classify-pdf tests/fixtures/tier1/simple.pdf

# Should output tier classification, not "Pdfium library not found"
```

### Troubleshooting

**Error: "Pdfium library not found"**
- Ensure pdfium.dll (Windows), libpdfium.dylib (macOS), or libpdfium.so (Linux) is:
  - In project root directory, OR
  - In directory specified by PDFIUM_LIB_PATH environment variable
- Check file permissions (must be readable/executable)

**Error: "pdfinfo: command not found"**
- classify-pdf will still work, but without metadata scoring
- Install poppler-utils to enable full functionality

**Build errors about pdfium-render**
- Ensure Rust version ≥ 1.82
- Try: `cargo clean && cargo build`
