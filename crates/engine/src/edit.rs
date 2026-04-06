//! Phase 4 edit engine — surgical AST mutation.
//!
//! This module implements all structural edit operations on a [`ParsedDocument`]:
//! delete, replace, and insert_after.  Each mutating operation is followed by a
//! deterministic renumbering pass that brings all sibling markers at the affected
//! level back into CSI canonical form.
//!
//! # Entry point
//!
//! Use [`SectionEditor`] to apply an [`EditRequest`] to a [`ParsedDocument`].
//!
//! ```ignore
//! let mut editor = SectionEditor::new(doc);
//! let result = editor.apply(request)?;
//! let updated_doc = editor.into_document();
//! ```
//!
//! # Addressing
//!
//! Nodes are addressed via a [`NodePath`]: a section ID and a marker sequence.
//! Example: `section_id="23 82 16"`, `markers=["PART 2", "2.7", "A."]` locates
//! the `A.` Paragraph inside Article 2.7 inside Part 2 of section 23 82 16.
//!
//! # Renumbering scheme (CSI 3-part)
//!
//! | Level | Format | Example |
//! |-------|--------|---------|
//! | 0 (Part) | `PART N` | `PART 1`, `PART 2` |
//! | 1 (Article) | `P.N` | `2.1`, `2.2` (P from parent Part) |
//! | 2 (Paragraph) | `A.` | `A.`, `B.`, `Z.` (max 26) |
//! | 3 (SubParagraph) | `N.` | `1.`, `2.` |
//! | 4 (SubSubParagraph) | `a.` | `a.`, `b.`, `z.` (max 26) |
//! | 5 (SubSubSubParagraph) | `N)` | `1)`, `2)` |

use conset_pdf_ir::{
    AstNode, EditError, EditOperation, EditRequest, EditResult, NodePath, ParsedDocument,
    SectionAst,
};

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Extracts the Part number from a Part marker like `"PART 2"` → `2`.
/// Returns `1` if the marker cannot be parsed (safe fallback).
fn part_number_from_marker(marker: &str) -> u32 {
    marker
        .split_whitespace()
        .nth(1)
        .and_then(|n| n.parse().ok())
        .unwrap_or(1)
}

/// Generates the canonical marker string for a node at the given level and
/// 1-based sibling position.
///
/// `parent_part_num` is only meaningful at level 1 (Article), where it supplies
/// the `P` in `P.N`.  For all other levels it is ignored.
fn make_marker(level: u8, one_based_pos: usize, parent_part_num: u32) -> Result<String, EditError> {
    match level {
        0 => Ok(format!("PART {one_based_pos}")),
        1 => Ok(format!("{parent_part_num}.{one_based_pos}")),
        2 => {
            if one_based_pos > 26 {
                return Err(EditError::RenumberOverflow { level, sibling_count: one_based_pos });
            }
            let ch = char::from(b'A' + (one_based_pos as u8) - 1);
            Ok(format!("{ch}."))
        }
        3 => Ok(format!("{one_based_pos}.")),
        4 => {
            if one_based_pos > 26 {
                return Err(EditError::RenumberOverflow { level, sibling_count: one_based_pos });
            }
            let ch = char::from(b'a' + (one_based_pos as u8) - 1);
            Ok(format!("{ch}."))
        }
        5 => Ok(format!("{one_based_pos})")),
        _ => Ok(format!("{one_based_pos}")),
    }
}

/// Renumbers all nodes in `siblings` using the canonical CSI marker scheme for
/// `level`.  `parent_marker` is the parent node's `marker` field and is used
/// to extract the Part number when `level == 1`.
///
/// Returns `Err` only if the sibling count would overflow the marker space.
pub fn renumber_siblings(
    siblings: &mut Vec<AstNode>,
    level: u8,
    parent_marker: &str,
) -> Result<(), EditError> {
    let parent_part_num = if level == 1 {
        part_number_from_marker(parent_marker)
    } else {
        0
    };
    for (idx, node) in siblings.iter_mut().enumerate() {
        node.marker = make_marker(level, idx + 1, parent_part_num)?;
    }
    Ok(())
}

/// Locates the target marker at the tip of `markers` inside `nodes`,
/// returning a mutable reference to the *parent* children list and the
/// 0-based index of the target within that list, along with the parent's
/// marker string (used for article renumbering context).
///
/// When `markers` has a single element, the parent list is `nodes` itself
/// and the parent marker is `""` (root).
fn find_in_parent<'a>(
    nodes: &'a mut Vec<AstNode>,
    markers: &[&str],
) -> Option<(String, &'a mut Vec<AstNode>, usize)> {
    if markers.is_empty() {
        return None;
    }

    // Single marker: target lives directly in `nodes`.
    if markers.len() == 1 {
        let leaf = markers[0];
        let idx = nodes.iter().position(|n| n.marker == leaf)?;
        return Some((String::new(), nodes, idx));
    }

    let head = markers[0];
    let rest = &markers[1..];
    let parent_idx = nodes.iter().position(|n| n.marker == head)?;

    if rest.len() == 1 {
        // One hop deeper: target is in head's children.
        let parent_marker = nodes[parent_idx].marker.clone();
        let leaf = rest[0];
        let leaf_idx =
            nodes[parent_idx].children.iter().position(|n| n.marker == leaf)?;
        return Some((parent_marker, &mut nodes[parent_idx].children, leaf_idx));
    }

    // More than two hops: recurse into head's children.
    find_in_parent(&mut nodes[parent_idx].children, rest)
}

/// Locates a target node for in-place mutation (replace).
fn find_node_mut<'a>(nodes: &'a mut Vec<AstNode>, markers: &[&str]) -> Option<&'a mut AstNode> {
    match markers {
        [] => None,
        [leaf] => nodes.iter_mut().find(|n| n.marker.as_str() == *leaf),
        [head, rest @ ..] => {
            let parent = nodes.iter_mut().find(|n| n.marker.as_str() == *head)?;
            find_node_mut(&mut parent.children, rest)
        }
    }
}

/// Immutable lookup — used in pre-flight validation.
fn find_node<'a>(nodes: &'a [AstNode], markers: &[&str]) -> Option<&'a AstNode> {
    match markers {
        [] => None,
        [leaf] => nodes.iter().find(|n| n.marker.as_str() == *leaf),
        [head, rest @ ..] => {
            let parent = nodes.iter().find(|n| n.marker.as_str() == *head)?;
            find_node(&parent.children, rest)
        }
    }
}

// ── Section-level operations ──────────────────────────────────────────────────

/// Validates that `path` resolves to an existing node in `section`.
fn preflight_path(section: &SectionAst, path: &NodePath) -> Result<(), EditError> {
    let markers: Vec<&str> = path.markers.iter().map(String::as_str).collect();
    if find_node(&section.nodes, &markers).is_none() {
        return Err(EditError::PathNotFound {
            section_id: path.section_id.clone(),
            markers: path.markers.clone(),
        });
    }
    Ok(())
}

/// Applies a Delete operation to `section`.
fn apply_delete_to_section(section: &mut SectionAst, path: &NodePath) -> Result<AstNode, EditError> {
    let markers: Vec<&str> = path.markers.iter().map(String::as_str).collect();
    let (parent_marker, siblings, idx) =
        find_in_parent(&mut section.nodes, &markers).ok_or_else(|| {
            EditError::PathNotFound {
                section_id: path.section_id.clone(),
                markers: path.markers.clone(),
            }
        })?;
    let removed = siblings.remove(idx);
    let level = removed.level;
    renumber_siblings(siblings, level, &parent_marker)?;
    Ok(removed)
}

/// Applies a Replace operation to `section` (text swap only; marker/position unchanged).
fn apply_replace_to_section(
    section: &mut SectionAst,
    path: &NodePath,
    new_text: &str,
) -> Result<(), EditError> {
    let markers: Vec<&str> = path.markers.iter().map(String::as_str).collect();
    let node = find_node_mut(&mut section.nodes, &markers).ok_or_else(|| {
        EditError::PathNotFound {
            section_id: path.section_id.clone(),
            markers: path.markers.clone(),
        }
    })?;
    node.text = new_text.to_string();
    Ok(())
}

/// Applies an InsertAfter operation to `section`.
fn apply_insert_after_to_section(
    section: &mut SectionAst,
    path: &NodePath,
    new_node: AstNode,
) -> Result<(), EditError> {
    // Level validation: new_node must match the sibling level at the target.
    let markers: Vec<&str> = path.markers.iter().map(String::as_str).collect();
    let target_level = {
        let target = find_node(&section.nodes, &markers).ok_or_else(|| {
            EditError::PathNotFound {
                section_id: path.section_id.clone(),
                markers: path.markers.clone(),
            }
        })?;
        target.level
    };
    if new_node.level != target_level {
        return Err(EditError::LevelMismatch {
            expected_level: target_level,
            got_level: new_node.level,
        });
    }

    let (parent_marker, siblings, idx) =
        find_in_parent(&mut section.nodes, &markers).ok_or_else(|| {
            EditError::PathNotFound {
                section_id: path.section_id.clone(),
                markers: path.markers.clone(),
            }
        })?;
    siblings.insert(idx + 1, new_node);
    renumber_siblings(siblings, target_level, &parent_marker)?;
    Ok(())
}

// ── SectionEditor (public API) ────────────────────────────────────────────────

/// Applies surgical edit operations to a [`ParsedDocument`].
///
/// # Usage
///
/// ```ignore
/// let mut editor = SectionEditor::new(doc);
/// let result = editor.apply(request);
/// let updated_doc = editor.into_document();
/// ```
pub struct SectionEditor {
    doc: ParsedDocument,
}

impl SectionEditor {
    /// Creates a new editor wrapping `doc`.
    pub fn new(doc: ParsedDocument) -> Self {
        Self { doc }
    }

    /// Consumes the editor and returns the (potentially modified) document.
    pub fn into_document(self) -> ParsedDocument {
        self.doc
    }

    /// Applies all operations in `request` to the document.
    ///
    /// Pre-flight validation runs over all operations before any mutation.
    /// If validation passes the operations are applied in order; the first
    /// runtime error stops execution and returns a failure result with the
    /// count of successfully applied operations.
    pub fn apply(&mut self, request: EditRequest) -> EditResult {
        if request.operations.is_empty() {
            return EditResult::err(0, EditError::EmptyRequest);
        }

        // ── Pre-flight: validate all paths before touching the document ────────
        for op in &request.operations {
            if let Err(e) = self.preflight(op) {
                return EditResult::err(0, e);
            }
        }

        // ── Apply in order ────────────────────────────────────────────────────
        let mut applied = 0;
        let mut warnings: Vec<String> = vec![];
        for op in request.operations {
            match self.apply_one(op) {
                Ok(w) => {
                    warnings.extend(w);
                    applied += 1;
                }
                Err(e) => {
                    return EditResult::err(applied, e);
                }
            }
        }

        EditResult::ok(applied, warnings)
    }

    /// Pre-flight validation for a single operation (no mutation).
    fn preflight(&self, op: &EditOperation) -> Result<(), EditError> {
        match op {
            EditOperation::Delete { path }
            | EditOperation::Replace { path, .. }
            | EditOperation::InsertAfter { path, .. } => {
                let section = self.find_section(&path.section_id)?;
                preflight_path(section, path)?;
                // InsertAfter: also validate level match.
                if let EditOperation::InsertAfter { new_node, .. } = op {
                    let markers: Vec<&str> =
                        path.markers.iter().map(String::as_str).collect();
                    let target_level =
                        find_node(&section.nodes, &markers).map(|n| n.level).unwrap_or(0);
                    if new_node.level != target_level {
                        return Err(EditError::LevelMismatch {
                            expected_level: target_level,
                            got_level: new_node.level,
                        });
                    }
                }
                Ok(())
            }
        }
    }

    /// Applies a single operation — caller guarantees pre-flight already passed.
    fn apply_one(&mut self, op: EditOperation) -> Result<Vec<String>, EditError> {
        match op {
            EditOperation::Delete { path } => {
                let section = self.find_section_mut(&path.section_id)?;
                apply_delete_to_section(section, &path)?;
                Ok(vec![])
            }
            EditOperation::Replace { path, new_text } => {
                let section = self.find_section_mut(&path.section_id)?;
                apply_replace_to_section(section, &path, &new_text)?;
                Ok(vec![])
            }
            EditOperation::InsertAfter { path, new_node } => {
                let section = self.find_section_mut(&path.section_id)?;
                apply_insert_after_to_section(section, &path, new_node)?;
                Ok(vec![])
            }
        }
    }

    fn find_section(&self, section_id: &str) -> Result<&SectionAst, EditError> {
        self.doc.sections.iter().find(|s| s.section_id == section_id).ok_or_else(|| {
            EditError::SectionNotFound { section_id: section_id.to_string() }
        })
    }

    fn find_section_mut(&mut self, section_id: &str) -> Result<&mut SectionAst, EditError> {
        self.doc.sections.iter_mut().find(|s| s.section_id == section_id).ok_or_else(|| {
            EditError::SectionNotFound { section_id: section_id.to_string() }
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{EditOperation, EditRequest, NodePath, OutlineTag};

    // ── Fixture builders ──────────────────────────────────────────────────────

    fn node(tag: OutlineTag, marker: &str, text: &str, level: u8, children: Vec<AstNode>) -> AstNode {
        AstNode { tag, marker: marker.into(), text: text.into(), page_index: 0, level, children }
    }

    fn part(num: u32, children: Vec<AstNode>) -> AstNode {
        node(OutlineTag::Part, &format!("PART {num}"), &format!("Part {num} heading"), 0, children)
    }

    fn article(major: u32, minor: u32, children: Vec<AstNode>) -> AstNode {
        node(
            OutlineTag::Article,
            &format!("{major}.{minor}"),
            &format!("Article {major}.{minor} text"),
            1,
            children,
        )
    }

    fn paragraph(letter: char, children: Vec<AstNode>) -> AstNode {
        node(
            OutlineTag::Paragraph,
            &format!("{letter}."),
            &format!("Paragraph {letter} text"),
            2,
            children,
        )
    }

    fn sub_para(n: usize, children: Vec<AstNode>) -> AstNode {
        node(OutlineTag::SubParagraph, &format!("{n}."), &format!("SubParagraph {n} text"), 3, children)
    }

    /// Builds a small three-part section.
    ///
    /// ```
    /// PART 1
    ///   1.1
    ///     A. (children: 1., 2.)
    ///     B.
    /// PART 2
    ///   2.1
    ///     A.
    ///     B.
    ///     C.
    ///   2.2
    /// PART 3
    ///   3.1
    /// ```
    fn sample_section() -> SectionAst {
        SectionAst {
            section_id: "23 82 16".to_string(),
            section_title: "Heating Water Coils".to_string(),
            start_page: 0,
            end_page: 5,
            nodes: vec![
                part(1, vec![article(
                    1, 1,
                    vec![
                        paragraph('A', vec![sub_para(1, vec![]), sub_para(2, vec![])]),
                        paragraph('B', vec![]),
                    ],
                )]),
                part(2, vec![
                    article(2, 1, vec![
                        paragraph('A', vec![]),
                        paragraph('B', vec![]),
                        paragraph('C', vec![]),
                    ]),
                    article(2, 2, vec![]),
                ]),
                part(3, vec![article(3, 1, vec![])]),
            ],
            parse_warnings: vec![],
        }
    }

    fn sample_doc() -> ParsedDocument {
        ParsedDocument {
            source_path: "/test/doc.pdf".to_string(),
            sections: vec![sample_section()],
            global_warnings: vec![],
        }
    }

    // ── make_marker ───────────────────────────────────────────────────────────

    #[test]
    fn make_marker_part() {
        assert_eq!(make_marker(0, 1, 0).unwrap(), "PART 1");
        assert_eq!(make_marker(0, 3, 0).unwrap(), "PART 3");
    }

    #[test]
    fn make_marker_article() {
        assert_eq!(make_marker(1, 1, 2).unwrap(), "2.1");
        assert_eq!(make_marker(1, 7, 2).unwrap(), "2.7");
    }

    #[test]
    fn make_marker_paragraph() {
        assert_eq!(make_marker(2, 1, 0).unwrap(), "A.");
        assert_eq!(make_marker(2, 3, 0).unwrap(), "C.");
        assert_eq!(make_marker(2, 26, 0).unwrap(), "Z.");
        assert!(make_marker(2, 27, 0).is_err());
    }

    #[test]
    fn make_marker_sub_paragraph() {
        assert_eq!(make_marker(3, 1, 0).unwrap(), "1.");
        assert_eq!(make_marker(3, 9, 0).unwrap(), "9.");
    }

    #[test]
    fn make_marker_sub_sub_paragraph() {
        assert_eq!(make_marker(4, 1, 0).unwrap(), "a.");
        assert_eq!(make_marker(4, 3, 0).unwrap(), "c.");
        assert!(make_marker(4, 27, 0).is_err());
    }

    #[test]
    fn make_marker_sub_sub_sub_paragraph() {
        assert_eq!(make_marker(5, 1, 0).unwrap(), "1)");
        assert_eq!(make_marker(5, 5, 0).unwrap(), "5)");
    }

    // ── renumber_siblings ─────────────────────────────────────────────────────

    #[test]
    fn renumber_paragraphs_after_delete() {
        let mut nodes =
            vec![paragraph('A', vec![]), paragraph('C', vec![]), paragraph('D', vec![])];
        // "B." was deleted — renumber A/C/D → A/B/C
        renumber_siblings(&mut nodes, 2, "").unwrap();
        let markers: Vec<&str> = nodes.iter().map(|n| n.marker.as_str()).collect();
        assert_eq!(markers, ["A.", "B.", "C."]);
    }

    #[test]
    fn renumber_articles_preserves_part_number() {
        let mut nodes = vec![article(2, 1, vec![]), article(2, 3, vec![])];
        // 2.2 deleted — renumber 2.1/2.3 → 2.1/2.2
        renumber_siblings(&mut nodes, 1, "PART 2").unwrap();
        let markers: Vec<&str> = nodes.iter().map(|n| n.marker.as_str()).collect();
        assert_eq!(markers, ["2.1", "2.2"]);
    }

    #[test]
    fn renumber_parts_sequential() {
        let mut nodes = vec![part(1, vec![]), part(3, vec![])];
        // PART 2 deleted — renumber → PART 1 / PART 2
        renumber_siblings(&mut nodes, 0, "").unwrap();
        let markers: Vec<&str> = nodes.iter().map(|n| n.marker.as_str()).collect();
        assert_eq!(markers, ["PART 1", "PART 2"]);
    }

    #[test]
    fn renumber_noop_when_already_correct() {
        let mut nodes =
            vec![paragraph('A', vec![]), paragraph('B', vec![]), paragraph('C', vec![])];
        renumber_siblings(&mut nodes, 2, "").unwrap();
        let markers: Vec<&str> = nodes.iter().map(|n| n.marker.as_str()).collect();
        assert_eq!(markers, ["A.", "B.", "C."]);
    }

    // ── find_node (immutable) ─────────────────────────────────────────────────

    #[test]
    fn find_node_finds_top_level() {
        let section = sample_section();
        let found = find_node(&section.nodes, &["PART 2"]);
        assert!(found.is_some());
        assert_eq!(found.unwrap().marker, "PART 2");
    }

    #[test]
    fn find_node_finds_deep_path() {
        let section = sample_section();
        let found = find_node(&section.nodes, &["PART 2", "2.1", "B."]);
        assert!(found.is_some());
        assert_eq!(found.unwrap().marker, "B.");
    }

    #[test]
    fn find_node_returns_none_for_wrong_section() {
        let section = sample_section();
        let found = find_node(&section.nodes, &["PART 9", "9.1"]);
        assert!(found.is_none());
    }

    #[test]
    fn find_node_returns_none_for_partial_bad_path() {
        let section = sample_section();
        let found = find_node(&section.nodes, &["PART 2", "2.1", "Z."]);
        assert!(found.is_none());
    }

    // ── Task 4.3: delete ──────────────────────────────────────────────────────

    #[test]
    fn delete_middle_sibling_renumbers() {
        let mut section = sample_section();
        let path = NodePath::new("23 82 16", vec!["PART 2", "2.1", "B."]);
        let removed = apply_delete_to_section(&mut section, &path).unwrap();
        assert_eq!(removed.marker, "B.");
        // Should now be A. C. → renumbered to A. B.
        let para_markers: Vec<&str> = section.nodes[1].children[0]
            .children
            .iter()
            .map(|n| n.marker.as_str())
            .collect();
        assert_eq!(para_markers, ["A.", "B."]);
    }

    #[test]
    fn delete_leaf_node() {
        let mut section = sample_section();
        let path = NodePath::new("23 82 16", vec!["PART 1", "1.1", "A.", "2."]);
        apply_delete_to_section(&mut section, &path).unwrap();
        let sub_markers: Vec<&str> = section.nodes[0].children[0].children[0]
            .children
            .iter()
            .map(|n| n.marker.as_str())
            .collect();
        // Was [1., 2.] → removed 2. → [1.]
        assert_eq!(sub_markers, ["1."]);
    }

    #[test]
    fn delete_top_level_part() {
        let mut section = sample_section();
        let path = NodePath::new("23 82 16", vec!["PART 2"]);
        apply_delete_to_section(&mut section, &path).unwrap();
        // PART 1, PART 3 → renumbered to PART 1, PART 2
        let part_markers: Vec<&str> = section.nodes.iter().map(|n| n.marker.as_str()).collect();
        assert_eq!(part_markers, ["PART 1", "PART 2"]);
    }

    // ── Task 4.5: replace ─────────────────────────────────────────────────────

    #[test]
    fn replace_updates_text_only() {
        let mut section = sample_section();
        let path = NodePath::new("23 82 16", vec!["PART 2", "2.1", "B."]);
        apply_replace_to_section(&mut section, &path, "Updated B text.").unwrap();
        let node = find_node(&section.nodes, &["PART 2", "2.1", "B."]).unwrap();
        assert_eq!(node.text, "Updated B text.");
        assert_eq!(node.marker, "B."); // marker unchanged
    }

    #[test]
    fn replace_preserves_children() {
        let mut section = sample_section();
        let path = NodePath::new("23 82 16", vec!["PART 1", "1.1", "A."]);
        apply_replace_to_section(&mut section, &path, "New A text.").unwrap();
        let node = find_node(&section.nodes, &["PART 1", "1.1", "A."]).unwrap();
        assert_eq!(node.text, "New A text.");
        assert_eq!(node.children.len(), 2); // sub-paragraphs preserved
    }

    // ── Task 4.6: insert_after ────────────────────────────────────────────────

    #[test]
    fn insert_after_middle_renumbers_downstream() {
        let mut section = sample_section();
        // Insert after B. in PART 2 > 2.1 — making B./NEW/C. → B./C./D. after renumber
        let new_para = paragraph('X', vec![]); // marker is overwritten by renumber
        let path = NodePath::new("23 82 16", vec!["PART 2", "2.1", "B."]);
        apply_insert_after_to_section(&mut section, &path, new_para).unwrap();
        let para_markers: Vec<&str> = section.nodes[1].children[0]
            .children
            .iter()
            .map(|n| n.marker.as_str())
            .collect();
        // Was A. B. C. → inserted after B. → A. B. [new] C. → renumbered A./B./C./D.
        assert_eq!(para_markers, ["A.", "B.", "C.", "D."]);
    }

    #[test]
    fn insert_after_last_sibling() {
        let mut section = sample_section();
        let new_para = paragraph('Z', vec![]);
        let path = NodePath::new("23 82 16", vec!["PART 2", "2.1", "C."]);
        apply_insert_after_to_section(&mut section, &path, new_para).unwrap();
        let para_markers: Vec<&str> = section.nodes[1].children[0]
            .children
            .iter()
            .map(|n| n.marker.as_str())
            .collect();
        assert_eq!(para_markers, ["A.", "B.", "C.", "D."]);
    }

    #[test]
    fn insert_after_level_mismatch_fails() {
        let mut section = sample_section();
        let wrong_level_node = article(2, 99, vec![]); // level 1 — wrong for a paragraph slot
        let path = NodePath::new("23 82 16", vec!["PART 2", "2.1", "A."]);
        let result = apply_insert_after_to_section(&mut section, &path, wrong_level_node);
        assert!(matches!(result, Err(EditError::LevelMismatch { .. })));
    }

    // ── Task 4.7: SectionEditor ───────────────────────────────────────────────

    #[test]
    fn section_editor_applies_single_delete() {
        let doc = sample_doc();
        let mut editor = SectionEditor::new(doc);
        let req = EditRequest::new(
            "Remove C.",
            vec![EditOperation::Delete {
                path: NodePath::new("23 82 16", vec!["PART 2", "2.1", "C."]),
            }],
        );
        let result = editor.apply(req);
        assert!(result.success);
        assert_eq!(result.operations_applied, 1);
        // Check renumbering
        let doc = editor.into_document();
        let markers: Vec<&str> = doc.sections[0].nodes[1].children[0]
            .children
            .iter()
            .map(|n| n.marker.as_str())
            .collect();
        assert_eq!(markers, ["A.", "B."]);
    }

    #[test]
    fn section_editor_applies_multi_op_request() {
        let doc = sample_doc();
        let mut editor = SectionEditor::new(doc);
        let req = EditRequest::new(
            "Replace A then delete B",
            vec![
                EditOperation::Replace {
                    path: NodePath::new("23 82 16", vec!["PART 2", "2.1", "A."]),
                    new_text: "Replaced text.".to_string(),
                },
                EditOperation::Delete {
                    path: NodePath::new("23 82 16", vec!["PART 2", "2.1", "C."]),
                },
            ],
        );
        let result = editor.apply(req);
        assert!(result.success);
        assert_eq!(result.operations_applied, 2);
    }

    #[test]
    fn section_editor_empty_request_fails() {
        let doc = sample_doc();
        let mut editor = SectionEditor::new(doc);
        let req = EditRequest::new("empty", vec![]);
        let result = editor.apply(req);
        assert!(!result.success);
        assert!(matches!(result.error, Some(EditError::EmptyRequest)));
    }

    #[test]
    fn section_editor_invalid_section_fails_preflight() {
        let doc = sample_doc();
        let mut editor = SectionEditor::new(doc);
        let req = EditRequest::new(
            "Wrong section",
            vec![EditOperation::Delete {
                path: NodePath::new("99 99 99", vec!["PART 1"]),
            }],
        );
        let result = editor.apply(req);
        assert!(!result.success);
        assert!(matches!(result.error, Some(EditError::SectionNotFound { .. })));
        assert_eq!(result.operations_applied, 0); // no ops applied before preflight fail
    }

    #[test]
    fn section_editor_invalid_path_fails_preflight() {
        let doc = sample_doc();
        let mut editor = SectionEditor::new(doc);
        let req = EditRequest::new(
            "Bad path",
            vec![EditOperation::Delete {
                path: NodePath::new("23 82 16", vec!["PART 2", "2.1", "Z."]),
            }],
        );
        let result = editor.apply(req);
        assert!(!result.success);
        assert!(matches!(result.error, Some(EditError::PathNotFound { .. })));
        assert_eq!(result.operations_applied, 0);
    }
}
