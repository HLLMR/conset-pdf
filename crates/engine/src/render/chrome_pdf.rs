//! Chrome subprocess PDF renderer.
//!
//! Converts an HTML string to PDF bytes by:
//!
//! 1. Writing the HTML to a temporary file.
//! 2. Discovering the Chrome binary (`CHROME_PATH` env var → well-known system
//!    paths).
//! 3. Running Chrome in headless print-to-PDF mode as a subprocess.
//! 4. Reading the output PDF bytes and returning them.
//! 5. Cleaning up temp files on exit (best-effort).
//!
//! # Chrome version requirement
//!
//! CSS `@page` margin-box rules (running headers/footers, page counters) require
//! **Chrome 120 or later**.  Earlier versions will silently drop the
//! margin-box rules and produce a PDF without headers/footers, but the body
//! content is always rendered correctly.
//!
//! # Environment variable
//!
//! Set `CHROME_PATH` to override automatic binary discovery.
//!
//! ```text
//! CHROME_PATH=C:\Program Files\Google\Chrome\Application\chrome.exe
//! ```

use conset_pdf_ir::RenderError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Candidate binary names / absolute paths searched when `CHROME_PATH` is unset.
#[cfg(target_os = "windows")]
const CHROME_CANDIDATES: &[&str] = &[
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
    r"C:\Program Files\Chromium\Application\chrome.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
];

#[cfg(not(target_os = "windows"))]
const CHROME_CANDIDATES: &[&str] = &[
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium-browser",
    "/usr/bin/chromium",
    "/snap/bin/chromium",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
];

/// Render `html` to PDF bytes using a local Chrome/Chromium installation.
///
/// # Errors
///
/// - [`RenderError::ChromeNotFound`] — no Chrome binary located.
/// - [`RenderError::ChromeRenderFailed`] — Chrome process exited with non-zero
///   status.
/// - [`RenderError::Io`] — temp-file write or PDF read failed.
pub fn render_html_to_pdf(html: &str) -> Result<Vec<u8>, RenderError> {
    let chrome = find_chrome()?;

    // Write HTML to a temp file.
    let html_path = write_temp_html(html)?;
    let pdf_path = html_path.with_extension("pdf");

    let result = invoke_chrome(&chrome, &html_path, &pdf_path);

    // Best-effort cleanup of temp files.
    let _ = std::fs::remove_file(&html_path);

    match result {
        Ok(()) => {
            let bytes = std::fs::read(&pdf_path)?;
            let _ = std::fs::remove_file(&pdf_path);
            Ok(bytes)
        }
        Err(e) => {
            let _ = std::fs::remove_file(&pdf_path);
            Err(e)
        }
    }
}

/// Locate the Chrome binary.
///
/// Resolution order:
/// 1. `CHROME_PATH` environment variable.
/// 2. Well-known platform-specific paths (see [`CHROME_CANDIDATES`]).
pub fn find_chrome() -> Result<PathBuf, RenderError> {
    // 1. Env override.
    if let Ok(path) = std::env::var("CHROME_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 2. Well-known paths.
    for candidate in CHROME_CANDIDATES {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Ok(p);
        }
    }

    let searched = CHROME_CANDIDATES.join("\n  ");
    Err(RenderError::ChromeNotFound(searched.to_owned()))
}

/// Write `html` to a uniquely-named file inside the system temp directory.
fn write_temp_html(html: &str) -> Result<PathBuf, RenderError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let filename = format!("conset-render-{pid}-{nanos}.html");
    let path = std::env::temp_dir().join(filename);
    std::fs::write(&path, html.as_bytes())?;
    Ok(path)
}

/// Invoke Chrome to print `html_path` to `pdf_path`.
fn invoke_chrome(
    chrome: &Path,
    html_path: &Path,
    pdf_path: &Path,
) -> Result<(), RenderError> {
    // Convert html_path to a file:// URL.  On Windows, backslashes must become
    // forward slashes and the drive letter prefix needs three slashes.
    let html_url = path_to_file_url(html_path);

    let output = Command::new(chrome)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        .arg("--run-all-compositor-stages-before-draw")
        // Suppress Chrome's own system header/footer (date + URL) so our CSS
        // @page rules supply the headers and footers instead.
        .arg("--print-to-pdf-no-header")
        .arg(format!("--print-to-pdf={}", pdf_path.display()))
        .arg(html_url)
        .output()
        .map_err(|e| RenderError::Io(e))?;

    if !output.status.success() {
        let exit_code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(RenderError::ChromeRenderFailed { exit_code, stderr });
    }

    Ok(())
}

/// Convert a filesystem path to a `file://` URL string.
fn path_to_file_url(path: &Path) -> String {
    let raw = path.to_string_lossy();
    #[cfg(target_os = "windows")]
    {
        // "C:\foo\bar.html" → "file:///C:/foo/bar.html"
        let forward = raw.replace('\\', "/");
        format!("file:///{forward}")
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("file://{raw}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_chrome_returns_error_when_path_doesnt_exist() {
        // Override CHROME_PATH to a nonexistent path, then verify error.
        // We temporarily set the env var, then unset it via a scoped guard.
        // Because tests may run in parallel we use an unlikely sentinel path.
        let sentinel = if cfg!(target_os = "windows") {
            r"C:\does-not-exist-conset-test\chrome.exe"
        } else {
            "/does-not-exist-conset-test/chrome"
        };
        // Only assert if none of the stock candidates exist (CI may have Chrome).
        // If CHROME_CANDIDATES are all absent and sentinel is used, error is expected.
        let all_absent = CHROME_CANDIDATES.iter().all(|c| !PathBuf::from(c).exists());
        if all_absent {
            std::env::set_var("CHROME_PATH", sentinel);
            let result = find_chrome();
            std::env::remove_var("CHROME_PATH");
            assert!(
                matches!(result, Err(RenderError::ChromeNotFound(_))),
                "expected ChromeNotFound, got: {result:?}"
            );
        }
    }

    #[test]
    fn path_to_file_url_produces_file_scheme() {
        let p = PathBuf::from(if cfg!(target_os = "windows") {
            r"C:\tmp\test.html"
        } else {
            "/tmp/test.html"
        });
        let url = path_to_file_url(&p);
        assert!(url.starts_with("file://"), "url={url}");
        assert!(url.ends_with("test.html"), "url={url}");
    }

    /// Full render round-trip.  Requires Chrome 120+ to be installed.
    /// Run locally with: `cargo test -- --ignored cli_chrome_render_roundtrip`
    #[test]
    #[ignore]
    fn chrome_render_roundtrip() {
        let html = "<!DOCTYPE html><html><body><p>Hello PDF</p></body></html>";
        let bytes = render_html_to_pdf(html).expect("render failed");
        // PDF files start with "%PDF-"
        assert!(bytes.starts_with(b"%PDF-"), "expected PDF header");
        assert!(bytes.len() > 1024, "PDF suspiciously small: {} bytes", bytes.len());
    }
}
