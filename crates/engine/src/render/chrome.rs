//! Full-page HTML assembler — wraps a body fragment with `<html>`, `<head>`,
//! embedded CSS, and `@page` rules that drive headers/footers in print output.
//!
//! # CSS paged media support
//!
//! Chrome 120+ supports CSS `@page` margin boxes (`@top-right`, `@bottom-left`,
//! `@bottom-right`) including `counter(page)` and `counter(pages)` when printing
//! to PDF.  Earlier Chrome versions silently ignore the margin-box rules; the
//! body content is still rendered correctly.
//!
//! # Security
//!
//! All [`SpecChromeMetadata`] string values are HTML-escaped before being
//! interpolated into the CSS `content` property strings.  Single-quote
//! characters inside those strings are additionally backslash-escaped to avoid
//! breaking the CSS string literal.

use conset_pdf_ir::{PageSize, RenderConfig, SpecChromeMetadata};

/// Assemble a complete HTML document ready for headless-Chrome PDF printing.
///
/// `body_html` is the fragment produced by [`super::body::build_body_html`].
pub fn build_full_html(
    body_html: &str,
    chrome: &SpecChromeMetadata,
    config: &RenderConfig,
) -> String {
    let (page_w, page_h) = match config.page_size {
        PageSize::Letter => ("8.5in", "11in"),
        PageSize::A4 => ("210mm", "297mm"),
    };

    // Header line: "FIRM  |  PROJECT NAME"
    let header_text = build_header_text(chrome);
    // Footer left: "DATE  SECTION_ID – SECTION_TITLE"
    let footer_left = build_footer_left(chrome);

    // CSS content property strings must use single-quote delimiters; the
    // strings themselves must escape their internal single quotes as \27.
    let header_css = css_content_string(&header_text);
    let footer_left_css = css_content_string(&footer_left);

    let font = &config.font_family;
    let font_size = config.font_size_pt;

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<style>
/* ── Page layout ─────────────────────────────────────────────── */
@page {{
  size: {page_w} {page_h};
  margin: 1in 0.75in 0.85in 0.75in;

  /* Running header — project info (Chrome 120+ required) */
  @top-right {{
    content: {header_css};
    font-family: {font};
    font-size: 8pt;
    color: #444;
  }}

  /* Running footer left — date + section ID/title */
  @bottom-left {{
    content: {footer_left_css};
    font-family: {font};
    font-size: 8pt;
    color: #444;
  }}

  /* Running footer right — page counter */
  @bottom-right {{
    content: 'Page ' counter(page) ' of ' counter(pages);
    font-family: {font};
    font-size: 8pt;
    color: #444;
  }}
}}

/* ── Body typography ─────────────────────────────────────────── */
body {{
  font-family: {font};
  font-size: {font_size}pt;
  line-height: 1.4;
  color: #000;
  margin: 0;
  padding: 0;
}}

/* ── CSI outline levels ──────────────────────────────────────── */
.csi-body {{
  width: 100%;
}}

/* PART heading: bold, all-caps, small top margin */
.csi-part {{
  font-weight: bold;
  text-transform: uppercase;
  margin-top: 1em;
  margin-bottom: 0.25em;
  page-break-after: avoid;
}}

/* Article: bold marker + title, avoid breaking immediately after */
.csi-article {{
  font-weight: bold;
  margin-top: 0.75em;
  margin-bottom: 0.1em;
  page-break-after: avoid;
}}

/* Paragraph: hanging indent 0.25 in */
.csi-para {{
  margin-top: 0.2em;
  margin-bottom: 0.1em;
  padding-left: 0.25in;
  text-indent: -0.25in;
  page-break-inside: avoid;
}}

/* SubParagraph: hanging indent 0.50 in */
.csi-sub1 {{
  margin-top: 0.1em;
  margin-bottom: 0.05em;
  padding-left: 0.5in;
  text-indent: -0.25in;
  page-break-inside: avoid;
}}

/* SubSubParagraph: hanging indent 0.75 in */
.csi-sub2 {{
  margin-top: 0.1em;
  padding-left: 0.75in;
  text-indent: -0.25in;
  page-break-inside: avoid;
}}

/* SubSubSubParagraph: hanging indent 1.00 in */
.csi-sub3 {{
  margin-top: 0.1em;
  padding-left: 1in;
  text-indent: -0.25in;
  page-break-inside: avoid;
}}

/* Unclassified: body-width block, no special indent */
.csi-unclassified {{
  margin-top: 0.2em;
  padding-left: 0;
}}

.marker {{
  font-weight: normal;
}}
</style>
</head>
<body>
{body_html}
</body>
</html>
"#,
        page_w = page_w,
        page_h = page_h,
        header_css = header_css,
        footer_left_css = footer_left_css,
        font = font,
        font_size = font_size,
        body_html = body_html,
    )
}

/// Build the header content string: "FIRM  |  PROJECT NAME".
/// Empty fields are omitted gracefully.
fn build_header_text(chrome: &SpecChromeMetadata) -> String {
    match (chrome.firm.is_empty(), chrome.project_name.is_empty()) {
        (false, false) => format!("{}  |  {}", chrome.firm, chrome.project_name),
        (false, true) => chrome.firm.clone(),
        (true, false) => chrome.project_name.clone(),
        (true, true) => String::new(),
    }
}

/// Build the footer-left content: "DATE  SECTION_ID \u{2013} SECTION_TITLE".
/// Uses a space-separated fallback if fields are empty.
fn build_footer_left(chrome: &SpecChromeMetadata) -> String {
    let section_part = match (chrome.section_id.is_empty(), chrome.section_title.is_empty()) {
        (false, false) => format!("{} \u{2013} {}", chrome.section_id, chrome.section_title),
        (false, true) => chrome.section_id.clone(),
        (true, false) => chrome.section_title.clone(),
        (true, true) => String::new(),
    };
    match (chrome.date.is_empty(), section_part.is_empty()) {
        (false, false) => format!("{}  {}", chrome.date, section_part),
        (false, true) => chrome.date.clone(),
        (true, false) => section_part,
        (true, true) => String::new(),
    }
}

/// Wrap a plain text string as a CSS single-quoted `content` value.
///
/// - HTML-encodes `&`, `<`, `>` (safe in CSS string context but defensive)
/// - Escapes `\` as `\\`
/// - Escapes `'` as `\27 ` (CSS Unicode escape for single-quote)
fn css_content_string(text: &str) -> String {
    let mut buf = String::with_capacity(text.len() + 2);
    buf.push('\'');
    for ch in text.chars() {
        match ch {
            '\'' => buf.push_str("\\27 "),
            '\\' => buf.push_str("\\\\"),
            _ => buf.push(ch),
        }
    }
    buf.push('\'');
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{RenderConfig, SpecChromeMetadata};

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
    fn full_html_contains_doctype_and_head() {
        let html = build_full_html("<div>body</div>", &sample_chrome(), &RenderConfig::default());
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<head>"));
        assert!(html.contains("</head>"));
        assert!(html.contains("<body>"));
    }

    #[test]
    fn full_html_interpolates_chrome_fields() {
        let html = build_full_html("<div></div>", &sample_chrome(), &RenderConfig::default());
        assert!(html.contains("RWB Consulting Engineers"));
        assert!(html.contains("Lake Highlands High School"));
        assert!(html.contains("2025-10-17"));
        assert!(html.contains("23 82 16"));
        assert!(html.contains("Heating Water Coils"));
    }

    #[test]
    fn full_html_contains_page_counter_css() {
        let html = build_full_html("<div></div>", &sample_chrome(), &RenderConfig::default());
        assert!(html.contains("counter(page)"));
        assert!(html.contains("counter(pages)"));
    }

    #[test]
    fn css_content_string_wraps_in_single_quotes() {
        let out = css_content_string("hello");
        assert_eq!(out, "'hello'");
    }

    #[test]
    fn css_content_string_escapes_single_quote() {
        let out = css_content_string("it's");
        assert!(out.contains("\\27"));
        assert!(!out.contains("it's"));
    }

    #[test]
    fn empty_chrome_fields_produce_valid_html() {
        let chrome = SpecChromeMetadata::default();
        let html = build_full_html("<div></div>", &chrome, &RenderConfig::default());
        // Should not panic and should produce a valid HTML skeleton.
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn a4_page_size_uses_mm_units() {
        let mut cfg = RenderConfig::default();
        cfg.page_size = conset_pdf_ir::PageSize::A4;
        let html = build_full_html("<div></div>", &sample_chrome(), &cfg);
        assert!(html.contains("210mm"));
        assert!(html.contains("297mm"));
    }
}
