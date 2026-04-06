//! Edit operation types for Phase 4: surgical AST mutation.
//!
//! An [`EditRequest`] describes one or more ordered operations to apply to a
//! [`ParsedDocument`].  Each operation targets a node identified by a
//! [`NodePath`] — a section ID plus the sequence of outline marker strings
//! that uniquely locates the node within the section tree.
//!
//! # Addressing model
//!
//! ```text
//! section_id = "23 82 16"
//! markers    = ["PART 2", "2.7", "A."]
//!               ───────   ─────  ───
//!               level 0   lvl 1  lvl 2
//! ```
//!
//! Each marker string must match [`AstNode::marker`] exactly (as stored by the
//! parser).  An empty `markers` vec targets the section root node list itself,
//! which is only valid for [`EditOperation::InsertAfter`] on the synthetic root.
//!
//! # Renumbering
//!
//! After insert or delete, all siblings at the affected level are renumbered
//! deterministically using the CSI scheme:
//!
//! | Level | Scheme |
//! |-------|--------|
//! | 0 (Part) | `PART 1`, `PART 2`, … |
//! | 1 (Article) | `N.1`, `N.2`, … (N = parent Part number) |
//! | 2 (Paragraph) | `A.`, `B.`, `C.`, … |
//! | 3 (SubParagraph) | `1.`, `2.`, `3.`, … |
//! | 4 (SubSubParagraph) | `a.`, `b.`, `c.`, … |
//! | 5 (SubSubSubParagraph) | `1)`, `2)`, `3)`, … |

use serde::{Deserialize, Serialize};

// ── NodePath ──────────────────────────────────────────────────────────────────

/// Address of a single AST node within a document.
///
/// `markers` is the sequence of [`AstNode::marker`] strings from the section
/// root to the target node, e.g. `["PART 2", "2.7", "A."]`.  An empty
/// `markers` vec refers to the section root level (used only for
/// top-of-section inserts).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodePath {
    /// Canonical CSI section ID (e.g. `"23 82 16"`).
    pub section_id: String,
    /// Ordered sequence of [`AstNode::marker`] strings locating the target node.
    pub markers: Vec<String>,
}

impl NodePath {
    /// Convenience constructor.
    pub fn new(section_id: impl Into<String>, markers: Vec<impl Into<String>>) -> Self {
        Self {
            section_id: section_id.into(),
            markers: markers.into_iter().map(Into::into).collect(),
        }
    }
}

// ── EditOperation ─────────────────────────────────────────────────────────────

/// A single structural mutation on the AST.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EditOperation {
    /// Insert a new node as the next sibling after the node at `path`.
    /// The new node is then renumbered into its correct position along with
    /// all subsequent siblings.
    InsertAfter {
        /// Target node — the new node is inserted immediately after this.
        path: NodePath,
        /// The node to insert.  Its `marker` field is ignored and replaced by
        /// the renumbering pass; its `tag` and `level` must match the sibling
        /// level at the target path.
        new_node: crate::AstNode,
    },
    /// Delete the node at `path` and renumber the remaining siblings.
    Delete {
        /// Target node to remove.
        path: NodePath,
    },
    /// Replace the text content of the node at `path` without changing its
    /// structural position or marker.  Children are preserved unchanged.
    Replace {
        /// Target node whose text is to be replaced.
        path: NodePath,
        /// New text content (replaces [`AstNode::text`] only).
        new_text: String,
    },
}

// ── EditRequest ───────────────────────────────────────────────────────────────

/// A batch of ordered edit operations to apply to a [`ParsedDocument`].
///
/// Operations are applied in declaration order.  If any operation fails its
/// pre-flight validation, the entire request is rejected before any mutation
/// occurs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditRequest {
    /// Human-readable description of what this edit batch represents.
    pub description: String,
    /// Ordered list of operations to apply.
    pub operations: Vec<EditOperation>,
}

impl EditRequest {
    /// Constructs a new edit request.
    pub fn new(description: impl Into<String>, operations: Vec<EditOperation>) -> Self {
        Self { description: description.into(), operations }
    }
}

// ── EditResult ────────────────────────────────────────────────────────────────

/// Outcome of applying an [`EditRequest`] to a `ParsedDocument`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditResult {
    /// `true` if all operations were applied successfully.
    pub success: bool,
    /// Number of operations successfully applied.
    pub operations_applied: usize,
    /// Non-fatal issues encountered (e.g. renumbering skipped a gap).
    pub warnings: Vec<String>,
    /// Error that caused the request to fail, if any.
    pub error: Option<EditError>,
}

impl EditResult {
    /// Constructs a success result.
    pub fn ok(operations_applied: usize, warnings: Vec<String>) -> Self {
        Self { success: true, operations_applied, warnings, error: None }
    }

    /// Constructs a failure result.
    pub fn err(operations_applied: usize, error: EditError) -> Self {
        Self { success: false, operations_applied, warnings: vec![], error: Some(error) }
    }
}

// ── EditError ─────────────────────────────────────────────────────────────────

/// Errors returned by the edit engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EditError {
    /// The section ID in a [`NodePath`] does not exist in the document.
    SectionNotFound {
        section_id: String,
    },
    /// The marker sequence in a [`NodePath`] could not be resolved in the tree.
    PathNotFound {
        section_id: String,
        markers: Vec<String>,
    },
    /// The new node's tag/level does not match the sibling level at the target path.
    LevelMismatch {
        expected_level: u8,
        got_level: u8,
    },
    /// Renumbering would overflow the available marker space for the level
    /// (e.g. more than 26 uppercase-letter paragraphs).
    RenumberOverflow {
        level: u8,
        sibling_count: usize,
    },
    /// The operations list is empty.
    EmptyRequest,
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditError::SectionNotFound { section_id } => {
                write!(f, "section not found: \"{section_id}\"")
            }
            EditError::PathNotFound { section_id, markers } => {
                write!(
                    f,
                    "path not found in section \"{section_id}\": [{}]",
                    markers.join(" > ")
                )
            }
            EditError::LevelMismatch { expected_level, got_level } => {
                write!(
                    f,
                    "level mismatch: expected level {expected_level}, got {got_level}"
                )
            }
            EditError::RenumberOverflow { level, sibling_count } => {
                write!(
                    f,
                    "renumber overflow at level {level}: {sibling_count} siblings exceed marker capacity"
                )
            }
            EditError::EmptyRequest => write!(f, "edit request contains no operations"),
        }
    }
}

impl std::error::Error for EditError {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AstNode, OutlineTag};

    fn dummy_node(marker: &str, text: &str, level: u8) -> AstNode {
        AstNode {
            tag: OutlineTag::Paragraph,
            marker: marker.to_string(),
            text: text.to_string(),
            page_index: 0,
            level,
            children: vec![],
        }
    }

    #[test]
    fn node_path_round_trips_via_serde() {
        let path = NodePath::new("23 82 16", vec!["PART 2", "2.7", "A."]);
        let json = serde_json::to_string(&path).expect("serialize");
        let back: NodePath = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(path, back);
    }

    #[test]
    fn edit_operation_insert_after_round_trips() {
        let op = EditOperation::InsertAfter {
            path: NodePath::new("23 82 16", vec!["PART 2", "2.7", "A."]),
            new_node: dummy_node("B.", "New paragraph text.", 2),
        };
        let json = serde_json::to_string(&op).expect("serialize");
        let back: EditOperation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(op, back);
    }

    #[test]
    fn edit_operation_delete_round_trips() {
        let op = EditOperation::Delete {
            path: NodePath::new("23 82 16", vec!["PART 2", "2.7", "B."]),
        };
        let json = serde_json::to_string(&op).expect("serialize");
        let back: EditOperation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(op, back);
    }

    #[test]
    fn edit_operation_replace_round_trips() {
        let op = EditOperation::Replace {
            path: NodePath::new("23 82 16", vec!["PART 2", "2.7", "A."]),
            new_text: "Updated text content.".to_string(),
        };
        let json = serde_json::to_string(&op).expect("serialize");
        let back: EditOperation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(op, back);
    }

    #[test]
    fn edit_request_round_trips() {
        let req = EditRequest::new(
            "Add paragraph C after B",
            vec![EditOperation::InsertAfter {
                path: NodePath::new("23 82 16", vec!["PART 2", "2.7", "B."]),
                new_node: dummy_node("C.", "New sub-item.", 2),
            }],
        );
        let json = serde_json::to_string(&req).expect("serialize");
        let back: EditRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(req, back);
    }

    #[test]
    fn edit_result_ok_round_trips() {
        let r = EditResult::ok(3, vec!["minor note".to_string()]);
        let json = serde_json::to_string(&r).expect("serialize");
        let back: EditResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(r, back);
    }

    #[test]
    fn edit_result_err_round_trips() {
        let r = EditResult::err(
            0,
            EditError::SectionNotFound { section_id: "23 82 16".to_string() },
        );
        let json = serde_json::to_string(&r).expect("serialize");
        let back: EditResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(r, back);
    }

    #[test]
    fn edit_error_display_messages() {
        assert!(EditError::SectionNotFound { section_id: "23 82 16".into() }
            .to_string()
            .contains("23 82 16"));
        assert!(EditError::PathNotFound {
            section_id: "23 82 16".into(),
            markers: vec!["PART 2".into(), "2.7".into()],
        }
        .to_string()
        .contains("PART 2 > 2.7"));
        assert!(EditError::LevelMismatch { expected_level: 2, got_level: 3 }
            .to_string()
            .contains("expected level 2"));
        assert!(EditError::RenumberOverflow { level: 2, sibling_count: 27 }
            .to_string()
            .contains("27"));
        assert!(EditError::EmptyRequest.to_string().contains("no operations"));
    }
}
