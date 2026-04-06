//! HTML AST visualisation for Phase 3 parse output.
//!
//! Renders a [`ParsedDocument`] as a self-contained HTML file that uses
//! `<details>/<summary>` for collapsible tree nodes and embedded CSS for
//! colour-coded outline levels.  No JavaScript or external resources are
//! required.
//!
//! # Color scheme
//!
//! | Tag | Color |
//! |-----|-------|
//! | Part | Crimson |
//! | Article | Navy |
//! | Paragraph | DarkGreen |
//! | SubParagraph | Teal |
//! | SubSubParagraph | DimGray |
//! | SubSubSubParagraph | Gray |
//! | Unclassified | LightGray italic |

use crate::error::{EngineError, Result};
use conset_pdf_ir::{AstNode, OutlineTag, ParsedDocument, SectionAst};
use std::fmt::Write as FmtWrite;
use std::path::Path;

/// Renders `doc` as a self-contained HTML file written to `output_path`.
///
/// Parent directories are created if they do not exist.
///
/// # Errors
///
/// Returns [`EngineError`] if a directory cannot be created or the file
/// cannot be written.
pub fn render_ast_html(doc: &ParsedDocument, output_path: &Path) -> Result<()> {
    let html = generate_html(doc);

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(EngineError::Io)?;
        }
    }

    std::fs::write(output_path, html).map_err(EngineError::Io)?;
    log::debug!("Wrote AST HTML → {}", output_path.display());
    Ok(())
}

// ── HTML generation ───────────────────────────────────────────────────────────

fn generate_html(doc: &ParsedDocument) -> String {
    let mut buf = String::with_capacity(64 * 1024);

    let _ = write!(
        buf,
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>AST — {source}</title>
<style>
  body {{ font-family: 'Segoe UI', system-ui, sans-serif; font-size: 14px;
          background: #f8f8f8; color: #222; margin: 0; padding: 1.5rem; }}
  h1   {{ font-size: 1.1rem; margin: 0 0 0.5rem; color: #555; }}
  h2   {{ font-size: 1rem; margin: 1.5rem 0 0.3rem; border-bottom: 1px solid #ccc; padding-bottom: 0.2rem; }}
  details  {{ margin: 0.1rem 0 0.1rem 1.2rem; }}
  summary  {{ cursor: pointer; list-style: none; padding: 0.15rem 0; }}
  summary::-webkit-details-marker {{ display: none; }}
  summary::before {{ content: '▶ '; font-size: 0.6rem; color: #999; }}
  details[open] > summary::before {{ content: '▼ '; }}
  .leaf > summary::before {{ content: '· '; }}
  .marker-part   {{ color: #c0392b; font-weight: bold; font-size: 1rem; }}
  .marker-article {{ color: #1a237e; font-weight: bold; }}
  .marker-paragraph {{ color: #1b5e20; font-weight: 600; }}
  .marker-sub    {{ color: #00838f; }}
  .marker-subsub {{ color: #616161; }}
  .marker-subsubsub {{ color: #9e9e9e; }}
  .marker-unclassified {{ color: #bbb; font-style: italic; }}
  .text {{ margin-left: 0.3rem; }}
  .page {{ font-size: 0.7rem; color: #aaa; margin-left: 0.4rem; }}
  .warn {{ color: #b71c1c; font-style: italic; font-size: 0.85rem; }}
  .section-header {{ background: #eee; padding: 0.4rem 0.6rem; border-radius: 4px;
                     margin-bottom: 0.5rem; }}
</style>
</head>
<body>
<h1>Parsed AST — <code>{source}</code></h1>
"#,
        source = escape_html(&doc.source_path)
    );

    if !doc.global_warnings.is_empty() {
        for w in &doc.global_warnings {
            let _ = write!(buf, "<p class=\"warn\">⚠ {}</p>\n", escape_html(w));
        }
    }

    for section in &doc.sections {
        render_section(&mut buf, section);
    }

    buf.push_str("</body>\n</html>\n");
    buf
}

fn render_section(buf: &mut String, section: &SectionAst) {
    let _ = write!(
        buf,
        "<h2 class=\"section-header\">Section <code>{}</code>",
        escape_html(&section.section_id)
    );
    if !section.section_title.is_empty() {
        let _ = write!(buf, " — {}", escape_html(&section.section_title));
    }
    let _ = write!(
        buf,
        " <span class=\"page\">(pages {}–{})</span></h2>\n",
        section.start_page,
        section.end_page
    );

    if !section.parse_warnings.is_empty() {
        for w in &section.parse_warnings {
            let _ = write!(buf, "<p class=\"warn\">⚠ {}</p>\n", escape_html(w));
        }
    }

    if section.nodes.is_empty() {
        buf.push_str("<p class=\"warn\">No outline nodes detected in this section.</p>\n");
    } else {
        buf.push_str("<ul style=\"margin:0;padding:0;list-style:none\">\n");
        for node in &section.nodes {
            render_node(buf, node, 0);
        }
        buf.push_str("</ul>\n");
    }
}

fn render_node(buf: &mut String, node: &AstNode, _depth: usize) {
    let marker_class = tag_class(&node.tag);
    let is_leaf = node.children.is_empty();
    let indent = usize::from(node.level) * 20;

    let summary = if node.marker.is_empty() {
        format!(
            "<span class=\"{marker_class}\">{text}</span>",
            text = escape_html(&node.text)
        )
    } else {
        format!(
            "<span class=\"{marker_class}\">{marker}</span><span class=\"text\">{text}</span>",
            marker = escape_html(&node.marker),
            text = escape_html(&node.text)
        )
    };

    let page_badge = format!(
        "<span class=\"page\">p.{}</span>",
        node.page_index
    );

    if is_leaf {
        let _ = write!(
            buf,
            "<li style=\"margin-left:{indent}px\"><details class=\"leaf\"><summary>{summary}{page_badge}</summary></details></li>\n"
        );
    } else {
        let _ = write!(
            buf,
            "<li style=\"margin-left:{indent}px\"><details open><summary>{summary}{page_badge}</summary>\n<ul style=\"margin:0;padding:0;list-style:none\">\n"
        );
        for child in &node.children {
            render_node(buf, child, _depth + 1);
        }
        buf.push_str("</ul></details></li>\n");
    }
}

fn tag_class(tag: &OutlineTag) -> &'static str {
    match tag {
        OutlineTag::Part => "marker-part",
        OutlineTag::Article => "marker-article",
        OutlineTag::Paragraph => "marker-paragraph",
        OutlineTag::SubParagraph => "marker-sub",
        OutlineTag::SubSubParagraph => "marker-subsub",
        OutlineTag::SubSubSubParagraph => "marker-subsubsub",
        OutlineTag::Unclassified => "marker-unclassified",
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
