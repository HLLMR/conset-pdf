//! IR types for the Phase 5 section-regeneration pipeline.
//!
//! [`SpecChromeMetadata`] is the JSON contract read from `--chrome-metadata` when
//! invoking the `regenerate` CLI subcommand.  It carries the project/firm/section
//! fields required to populate headers and footers on the rendered PDF.
//!
//! [`RenderConfig`] controls font and paper settings.  Its default values target
//! a Letter-size spec sheet with 10 pt Arial body text — the most common AEC
//! spec format.
//!
//! [`RenderResult`] is returned by the engine's `SectionRenderer`.
//!
//! [`RenderError`] enumerates all failure modes in the render pipeline.

use serde::{Deserialize, Serialize};

/// Chrome (header/footer) metadata for a single spec section render.
///
/// Compose from [`crate::ChromeMetadata`] (global project fields) plus
/// [`crate::SectionEntry`] (section-specific fields) before passing to the
/// renderer.
///
/// All fields are plain strings.  Empty strings are safe; the renderer will
/// omit blank fields rather than printing an empty label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SpecChromeMetadata {
    /// Short project identifier (e.g. `"RWB Project No. 25063.00"`).
    pub project_id: String,
    /// Human-readable project / building name (e.g. `"Lake Highlands High School"`).
    pub project_name: String,
    /// CSI section ID (e.g. `"23 82 16"`).
    pub section_id: String,
    /// Section title as it appears in the spec book (e.g. `"Heating Water Coils"`).
    pub section_title: String,
    /// Revision / addendum date in any displayable format (e.g. `"2025-10-17"`).
    pub date: String,
    /// Engineering firm name (e.g. `"RWB Consulting Engineers"`).
    pub firm: String,
}

/// Paper size for rendered output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageSize {
    /// US Letter (8.5 × 11 in).  Default for North American AEC specs.
    Letter,
    /// ISO A4 (210 × 297 mm).
    A4,
}

impl Default for PageSize {
    fn default() -> Self {
        Self::Letter
    }
}

/// Font and layout settings for the rendered section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderConfig {
    /// CSS font-family string (e.g. `"Arial, sans-serif"`).
    pub font_family: String,
    /// Body text size in points.
    pub font_size_pt: u8,
    /// Paper size used for the PDF output.
    pub page_size: PageSize,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            font_family: "Arial, sans-serif".to_owned(),
            font_size_pt: 10,
            page_size: PageSize::Letter,
        }
    }
}

/// Successful output from `SectionRenderer::render`.
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// Raw PDF bytes — write directly to a `.pdf` file.
    pub pdf_bytes: Vec<u8>,
    /// Page count estimate (derived from PDF byte content; may be 0 on dry runs).
    pub page_count_estimate: u32,
    /// Non-fatal warnings emitted during rendering.
    pub warnings: Vec<String>,
}

/// Failure modes in the section-rendering pipeline.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// Chrome binary not found at `CHROME_PATH` env var or well-known system paths.
    #[error(
        "Chrome not found; set CHROME_PATH or install Chrome 120+.\nSearched paths:\n{0}"
    )]
    ChromeNotFound(String),

    /// HTML body or template assembly failed (programming error or malformed AST).
    #[error("HTML build error: {0}")]
    HtmlBuildError(String),

    /// Chrome subprocess exited with non-zero status.
    #[error("Chrome render failed (exit {exit_code}): {stderr}")]
    ChromeRenderFailed {
        /// Process exit code.
        exit_code: i32,
        /// Captured stderr from the Chrome process.
        stderr: String,
    },

    /// A file-system operation (temp file creation, PDF read) failed.
    #[error("I/O error during render: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_chrome_metadata_serde_round_trip() {
        let meta = SpecChromeMetadata {
            project_id: "RWB 25063.00".to_owned(),
            project_name: "Lake Highlands High School".to_owned(),
            section_id: "23 82 16".to_owned(),
            section_title: "Heating Water Coils".to_owned(),
            date: "2025-10-17".to_owned(),
            firm: "RWB Consulting Engineers".to_owned(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let restored: SpecChromeMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(meta, restored);
    }

    #[test]
    fn render_config_default_is_letter_10pt_arial() {
        let cfg = RenderConfig::default();
        assert_eq!(cfg.page_size, PageSize::Letter);
        assert_eq!(cfg.font_size_pt, 10);
        assert!(cfg.font_family.contains("Arial"));
    }

    #[test]
    fn render_config_serde_round_trip() {
        let cfg = RenderConfig {
            font_family: "Times New Roman, serif".to_owned(),
            font_size_pt: 11,
            page_size: PageSize::A4,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: RenderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, restored);
    }

    #[test]
    fn render_error_chrome_not_found_message_contains_searched_paths() {
        let err = RenderError::ChromeNotFound(
            "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_owned(),
        );
        let msg = err.to_string();
        assert!(msg.contains("CHROME_PATH"));
        assert!(msg.contains("chrome.exe"));
    }
}
