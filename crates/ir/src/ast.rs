//! AST types for Phase 3 paragraph parsing.
//!
//! A [`ParsedDocument`] is the output of the Parse stage: a hierarchical
//! Abstract Syntax Tree built from the line content of each CSI section.
//!
//! # CSI 3-Part outline levels
//!
//! | Level | Tag | Marker examples |
//! |-------|-----|-----------------|
//! | 0 | [`OutlineTag::Part`] | `PART 1`, `PART 2 — PRODUCTS` |
//! | 1 | [`OutlineTag::Article`] | `1.1`, `2.7` |
//! | 2 | [`OutlineTag::Paragraph`] | `A.`, `B.` |
//! | 3 | [`OutlineTag::SubParagraph`] | `1.`, `2.` |
//! | 4 | [`OutlineTag::SubSubParagraph`] | `a.`, `b.` |
//! | 5 | [`OutlineTag::SubSubSubParagraph`] | `1)`, `2)` |
//! | — | [`OutlineTag::Unclassified`] | (no recognizable marker) |

use serde::{Deserialize, Serialize};

fn default_font_name() -> String {
    "Unknown".to_string()
}

/// Classification of a node in the CSI 3-part section outline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutlineTag {
    /// Top-level section division: `PART 1 — GENERAL`, `PART 2 — PRODUCTS`, etc.
    Part,
    /// Numbered article within a Part: `1.1`, `2.7 HYDRONIC HEATING COILS`.
    Article,
    /// Uppercase-letter paragraph: `A.`, `B.`, `C.`.
    Paragraph,
    /// Numeric sub-paragraph (under a Paragraph): `1.`, `2.`, `3.`.
    SubParagraph,
    /// Lowercase-letter sub-sub-paragraph: `a.`, `b.`, `c.`.
    SubSubParagraph,
    /// Numeric sub-sub-sub-paragraph with closing paren: `1)`, `2)`.
    SubSubSubParagraph,
    /// Text that does not match any recognized outline pattern.
    Unclassified,
}

/// A single node in the document outline tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstNode {
    /// Semantic classification of this node.
    pub tag: OutlineTag,
    /// Outline marker as it appears in the text (e.g. `"PART 1"`, `"1.1"`, `"A."`, `"1."`, `"a."`).
    /// Empty string for [`OutlineTag::Unclassified`] nodes.
    pub marker: String,
    /// Text content following the marker (trimmed).  For continuation lines the
    /// text is appended to the last structural node so may span multiple visual lines.
    pub text: String,
    /// Zero-based page index in the source document where this node starts.
    pub page_index: usize,
    /// Nesting depth: 0 = Part, 1 = Article, 2 = Paragraph, 3 = SubParagraph,
    /// 4 = SubSubParagraph, 5 = SubSubSubParagraph.  Unclassified is 0.
    pub level: u8,
    /// Child nodes in the outline tree.
    pub children: Vec<AstNode>,
    /// Normalized x position (0.0–1.0) of the leftmost span on this node's first line.
    /// Used to compute per-level indentation for layout-geometry-aware rendering.
    /// Defaults to `0.0` for synthetic nodes and when deserializing older AST JSON files.
    #[serde(default)]
    pub x_indent: f64,
}

/// Measured layout geometry for a parsed section.
///
/// Computed from the raw span positions during parsing; used by the render
/// pipeline to produce accurately indented PDF output rather than relying on
/// hardcoded CSS values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionLayout {
    /// Normalized x coordinate (0.0–1.0) of the leftmost body span — i.e. the
    /// physical left margin of the section's text block on the page.
    pub body_left: f64,
    /// Normalized x coordinate of the rightmost body span's right edge.
    pub body_right: f64,
    /// Median font size of body spans, in points (original PDF scale).
    pub font_size_pt: f64,
    /// Median top-to-top y distance between consecutive lines, in normalized
    /// page coordinates.  Multiply by `11.0 * 72.0` to get points on Letter.
    pub line_gap_norm: f64,
    /// Modal (most-frequent) font family name among body spans, as reported by
    /// PDFium.  Used by the renderer to match the source document's typeface.
    /// Defaults to `"Unknown"` for sections with no extractable font metadata.
    #[serde(default = "default_font_name")]
    pub body_font_name: String,
}

/// The parsed AST for a single CSI section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionAst {
    /// Canonical CSI section ID (e.g. `"23 82 16"`).
    pub section_id: String,
    /// Section title as extracted from the segment index (empty string if unknown).
    pub section_title: String,
    /// Zero-based index of the first page in the source document.
    pub start_page: usize,
    /// Zero-based index of the last page in the source document (inclusive).
    pub end_page: usize,
    /// Top-level outline nodes for this section (usually Parts).
    pub nodes: Vec<AstNode>,
    /// Non-fatal issues encountered while parsing this section.
    pub parse_warnings: Vec<String>,
    /// Measured layout geometry for this section, or `None` if fewer than two
    /// body spans were found (e.g. empty or title-only sections).
    #[serde(default)]
    pub layout: Option<SectionLayout>,
}

/// The complete parsed AST for a document, covering all segmented sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedDocument {
    /// Absolute path of the source PDF (copied from `SegmentIndex.source_path`).
    pub source_path: String,
    /// ASTs for each detected section, in document order.
    pub sections: Vec<SectionAst>,
    /// Document-level warnings that apply across sections.
    pub global_warnings: Vec<String>,
}
