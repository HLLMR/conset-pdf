//! HTML body builder — converts a [`SectionAst`] to an HTML fragment.
//!
//! The output is a `<div class="csi-body">…</div>` block with one child element
//! per [`AstNode`].  All string content is HTML-escaped before insertion.
//!
//! # CSS class → CSI level mapping
//!
//! | Class | [`OutlineTag`] | Indent |
//! |-------|---------------|--------|
//! | `csi-part` | Part | 0 |
//! | `csi-article` | Article | 0 |
//! | `csi-para` | Paragraph | 0.25 in |
//! | `csi-sub1` | SubParagraph | 0.50 in |
//! | `csi-sub2` | SubSubParagraph | 0.75 in |
//! | `csi-sub3` | SubSubSubParagraph | 1.00 in |
//! | `csi-unclassified` | Unclassified | 0 |

use conset_pdf_ir::{AstNode, OutlineTag, RenderConfig, SectionAst};

/// Build an HTML fragment for the body content of one section.
///
/// Returns a `String` containing a `<div class="csi-body">` block.
/// The caller ([`super::chrome`]) wraps this inside a full HTML document
/// with `<html>`, `<head>`, and `<body>` tags.
pub fn build_body_html(ast: &SectionAst, _config: &RenderConfig) -> String {
    let mut buf = String::with_capacity(4096);
    buf.push_str("<div class=\"csi-body\">\n");
    for node in &ast.nodes {
        render_node(node, &mut buf);
    }
    buf.push_str("</div>\n");
    buf
}

fn render_node(node: &AstNode, buf: &mut String) {
    let (class, show_marker) = css_class_for(&node.tag);

    buf.push_str("<div class=\"");
    buf.push_str(class);
    buf.push_str("\">");

    if show_marker && !node.marker.is_empty() {
        buf.push_str("<span class=\"marker\">");
        push_escaped(&node.marker, buf);
        buf.push_str("</span> ");
    }

    push_escaped(&node.text, buf);

    // Recurse into children before closing the div so nesting is visible.
    if !node.children.is_empty() {
        buf.push('\n');
        for child in &node.children {
            render_node(child, buf);
        }
    }

    buf.push_str("</div>\n");
}

/// Returns `(css_class, show_marker)` for a given [`OutlineTag`].
fn css_class_for(tag: &OutlineTag) -> (&'static str, bool) {
    match tag {
        OutlineTag::Part => ("csi-part", false),
        OutlineTag::Article => ("csi-article", true),
        OutlineTag::Paragraph => ("csi-para", true),
        OutlineTag::SubParagraph => ("csi-sub1", true),
        OutlineTag::SubSubParagraph => ("csi-sub2", true),
        OutlineTag::SubSubSubParagraph => ("csi-sub3", true),
        OutlineTag::Unclassified => ("csi-unclassified", false),
    }
}

/// HTML-escape a string and append it to `buf`.
///
/// Escapes `&`, `<`, `>`, `"`, and `'`.
fn push_escaped(s: &str, buf: &mut String) {
    for ch in s.chars() {
        match ch {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            '"' => buf.push_str("&quot;"),
            '\'' => buf.push_str("&#39;"),
            other => buf.push(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::RenderConfig;

    fn make_node(tag: OutlineTag, marker: &str, text: &str) -> AstNode {
        AstNode {
            tag,
            marker: marker.to_owned(),
            text: text.to_owned(),
            page_index: 0,
            level: 0,
            children: vec![],
        }
    }

    fn make_ast(nodes: Vec<AstNode>) -> SectionAst {
        SectionAst {
            section_id: "23 82 16".to_owned(),
            section_title: "Heating Water Coils".to_owned(),
            start_page: 0,
            end_page: 1,
            nodes,
            parse_warnings: vec![],
        }
    }

    #[test]
    fn body_html_contains_part_text() {
        let ast = make_ast(vec![make_node(OutlineTag::Part, "PART 1", "GENERAL")]);
        let html = build_body_html(&ast, &RenderConfig::default());
        assert!(html.contains("csi-part"));
        assert!(html.contains("GENERAL"));
        // Part nodes do not show marker separately.
        assert!(!html.contains("<span class=\"marker\">PART 1</span>"));
    }

    #[test]
    fn body_html_shows_paragraph_marker() {
        let ast = make_ast(vec![make_node(OutlineTag::Paragraph, "A.", "Install per spec.")]);
        let html = build_body_html(&ast, &RenderConfig::default());
        assert!(html.contains("csi-para"));
        assert!(html.contains("<span class=\"marker\">A.</span>"));
        assert!(html.contains("Install per spec."));
    }

    #[test]
    fn html_special_chars_are_escaped() {
        let ast = make_ast(vec![make_node(
            OutlineTag::Unclassified,
            "",
            "5 < 10 & \"quoted\" or 'apos'",
        )]);
        let html = build_body_html(&ast, &RenderConfig::default());
        assert!(html.contains("5 &lt; 10 &amp; &quot;quoted&quot; or &#39;apos&#39;"));
        // The raw source characters must not appear in the text content.
        assert!(!html.contains("5 < 10"));
        assert!(!html.contains("&\"quoted\""));
    }

    #[test]
    fn body_html_nests_children() {
        let child = make_node(OutlineTag::SubParagraph, "1.", "Child text.");
        let mut parent = make_node(OutlineTag::Paragraph, "A.", "Parent text.");
        parent.children = vec![child];
        let ast = make_ast(vec![parent]);
        let html = build_body_html(&ast, &RenderConfig::default());
        // Both classes should appear.
        assert!(html.contains("csi-para"));
        assert!(html.contains("csi-sub1"));
        // Child content is inside the parent div (simple string containment sufficient).
        assert!(html.contains("Child text."));
    }
}
