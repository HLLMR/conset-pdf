//! Section regeneration render module.
//!
//! Orchestrates the three-step render pipeline:
//!
//! 1. [`body::build_body_html`] — AST → HTML body fragment.
//! 2. [`chrome::build_full_html`] — wrap body with `<head>`, CSS, and
//!    `@page` header/footer rules.
//! 3. [`chrome_pdf::render_html_to_pdf`] — invoke Chrome subprocess → PDF bytes.
//!
//! The public entry point is [`SectionRenderer`].

pub mod body;
pub mod chrome;
pub mod chrome_pdf;

use conset_pdf_ir::{RenderConfig, RenderError, RenderResult, SectionAst, SpecChromeMetadata};

/// Renders a single [`SectionAst`] to a PDF byte vector.
///
/// Requires a local Chrome 120+ installation reachable via `CHROME_PATH` or
/// a well-known system path.  See [`chrome_pdf`] for discovery details.
pub struct SectionRenderer {
    config: RenderConfig,
}

impl SectionRenderer {
    /// Create a renderer with the given configuration.
    #[must_use]
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Create a renderer with default configuration (Letter, 10pt Arial).
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(RenderConfig::default())
    }

    /// Render one section to PDF bytes.
    ///
    /// # Errors
    ///
    /// Propagates [`RenderError`] from any stage in the pipeline.
    pub fn render(
        &self,
        ast: &SectionAst,
        chrome_meta: &SpecChromeMetadata,
    ) -> Result<RenderResult, RenderError> {
        // Stage 1: AST → HTML body fragment.
        let body_html = body::build_body_html(ast, &self.config);

        // Stage 2: Wrap with <head>, CSS, @page rules.
        let full_html = chrome::build_full_html(&body_html, chrome_meta, &self.config);

        // Stage 3: HTML → PDF via Chrome subprocess.
        let pdf_bytes = chrome_pdf::render_html_to_pdf(&full_html)?;

        // Estimate page count from PDF byte content: count "Page" occurrences
        // in the xref table area is fragile; instead, count `/Page ` dict entries.
        let page_count_estimate = estimate_page_count(&pdf_bytes);

        Ok(RenderResult { pdf_bytes, page_count_estimate, warnings: vec![] })
    }

    /// Dry-run render: validate inputs and build the HTML but skip Chrome.
    ///
    /// Returns a [`RenderResult`] with empty `pdf_bytes` and a warning noting
    /// the dry-run.  Useful for argument validation in the CLI.
    pub fn dry_run(
        &self,
        ast: &SectionAst,
        chrome_meta: &SpecChromeMetadata,
    ) -> RenderResult {
        let body_html = body::build_body_html(ast, &self.config);
        let _full_html = chrome::build_full_html(&body_html, chrome_meta, &self.config);
        RenderResult {
            pdf_bytes: Vec::new(),
            page_count_estimate: 0,
            warnings: vec![
                "dry_run: HTML assembled successfully — Chrome render skipped".to_owned(),
            ],
        }
    }
}

/// Estimate the number of pages in a PDF by counting `/Type /Page` occurrences.
///
/// This is an approximation; a robust implementation would parse the xref table.
/// Sufficient for Phase 5 "Bondo doesn't show" quality.
fn estimate_page_count(pdf_bytes: &[u8]) -> u32 {
    // The pattern b"/Type /Page\n" or b"/Type/Page" appears once per page dict.
    let needle = b"/Type /Page";
    let mut count = 0u32;
    let mut pos = 0;
    while pos + needle.len() <= pdf_bytes.len() {
        if pdf_bytes[pos..].starts_with(needle) {
            count += 1;
            pos += needle.len();
        } else {
            pos += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{AstNode, OutlineTag};

    fn minimal_ast() -> SectionAst {
        SectionAst {
            section_id: "23 82 16".to_owned(),
            section_title: "Heating Water Coils".to_owned(),
            start_page: 0,
            end_page: 0,
            nodes: vec![AstNode {
                tag: OutlineTag::Part,
                marker: "PART 1".to_owned(),
                text: "GENERAL".to_owned(),
                page_index: 0,
                level: 0,
                children: vec![],
            }],
            parse_warnings: vec![],
        }
    }

    fn sample_chrome() -> SpecChromeMetadata {
        SpecChromeMetadata {
            project_id: "RWB 25063".to_owned(),
            project_name: "Lake Highlands High School".to_owned(),
            section_id: "23 82 16".to_owned(),
            section_title: "Heating Water Coils".to_owned(),
            date: "2025-10-17".to_owned(),
            firm: "RWB Consulting Engineers".to_owned(),
        }
    }

    #[test]
    fn dry_run_returns_empty_pdf_bytes_and_warning() {
        let renderer = SectionRenderer::with_defaults();
        let result = renderer.dry_run(&minimal_ast(), &sample_chrome());
        assert!(result.pdf_bytes.is_empty());
        assert_eq!(result.page_count_estimate, 0);
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("dry_run"));
    }

    #[test]
    fn estimate_page_count_returns_zero_for_empty() {
        assert_eq!(estimate_page_count(b""), 0);
    }

    #[test]
    fn estimate_page_count_finds_pages() {
        let mock_pdf = b"...obj /Type /Page endobj ...obj /Type /Page endobj...";
        assert_eq!(estimate_page_count(mock_pdf), 2);
    }

    /// End-to-end render test.  Requires Chrome 120+ installed locally.
    /// Run with: `cargo test -- --ignored render_section_produces_pdf`
    #[test]
    #[ignore]
    fn render_section_produces_pdf() {
        let renderer = SectionRenderer::with_defaults();
        let result = renderer
            .render(&minimal_ast(), &sample_chrome())
            .expect("render failed");
        assert!(result.pdf_bytes.starts_with(b"%PDF-"), "expected PDF header");
        assert!(result.page_count_estimate >= 1, "expected at least 1 page");
        assert!(
            result.pdf_bytes.len() > 1024,
            "PDF suspiciously small: {} bytes",
            result.pdf_bytes.len(),
        );
    }
}
