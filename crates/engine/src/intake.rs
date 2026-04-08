//! Stage 0 Intake Triage engine — rotation detection and normalization.
//!
//! Loads each PDF in an [`IntakeBundle`], detects pages whose `/Rotate`
//! value is not a multiple of 360°, reports them as [`IntakeIssue`]s, and
//! (unless dry-run) normalizes each such page to 0° and saves the file
//! back in-place.

use conset_pdf_contracts::intake::{IntakeBundle, IntakeIssue, IssueSeverity};
use lopdf::{Document, Object, ObjectId};
use std::collections::BTreeMap;

// ── Public API ────────────────────────────────────────────────────────────────

/// Result for a single file processed by [`Stage0Normalizer`].
#[derive(Debug)]
pub struct FileNormalizationResult {
    /// Filesystem path of the processed file.
    pub path: String,
    /// Total number of pages detected in the PDF.
    pub page_count: u32,
    /// Issues found during normalization (rotated pages, corrupt body, etc.).
    pub issues: Vec<IntakeIssue>,
    /// Number of pages whose rotation was corrected on disk (0 in dry-run mode).
    pub rotations_normalized: usize,
}

/// Combined output of [`Stage0Normalizer::normalize`] for an [`IntakeBundle`].
#[derive(Debug)]
pub struct NormalizationResult {
    /// Per-file results in the same order as [`IntakeBundle::files`].
    pub files: Vec<FileNormalizationResult>,
}

/// Stateless processor for Stage 0 intake triage.
///
/// Loads each PDF in an [`IntakeBundle`], detects rotated pages, and
/// optionally writes rotation-normalized copies back in-place.
pub struct Stage0Normalizer;

impl Stage0Normalizer {
    /// Process every file in `intake`.
    ///
    /// When `dry_run` is `true` rotation issues are still detected and reported,
    /// but no files are modified on disk.
    pub fn normalize(intake: &IntakeBundle, dry_run: bool) -> NormalizationResult {
        let files = intake.files.iter().map(|f| normalize_one(&f.path, dry_run)).collect();
        NormalizationResult { files }
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn normalize_one(path: &str, dry_run: bool) -> FileNormalizationResult {
    let mut doc = match Document::load(path) {
        Ok(d) => d,
        Err(e) => {
            return FileNormalizationResult {
                path: path.to_owned(),
                page_count: 0,
                issues: vec![IntakeIssue {
                    issue_id: format!("corrupt-{}", sanitize_id(path)),
                    severity: IssueSeverity::Fatal,
                    code: "CORRUPT_PDF".to_owned(),
                    description: format!("Cannot load PDF '{path}': {e}"),
                    page_index: None,
                    suggested_action: Some(
                        "Verify the file is a valid, unencrypted PDF.".to_owned(),
                    ),
                }],
                rotations_normalized: 0,
            };
        }
    };

    let pages = doc.get_pages();
    let page_count = pages.len() as u32;
    let rotated = detect_rotations(&doc, &pages);

    let issues: Vec<IntakeIssue> = rotated
        .iter()
        .map(|(page_num, degrees)| IntakeIssue {
            issue_id: format!("rotate-p{page_num}"),
            severity: IssueSeverity::Warning,
            code: "ROTATED_PAGE".to_owned(),
            description: format!("Page {page_num} is rotated {degrees}°"),
            page_index: Some((*page_num as usize).saturating_sub(1)),
            suggested_action: Some(
                "Rotation will be normalized to 0° automatically.".to_owned(),
            ),
        })
        .collect();

    let rotations_normalized = if dry_run || rotated.is_empty() {
        0
    } else {
        let page_ids: Vec<ObjectId> = rotated.iter().map(|(n, _)| pages[n]).collect();
        apply_normalization(&mut doc, &page_ids);
        match doc.save(path) {
            Ok(_) => rotated.len(),
            Err(e) => {
                // Return 0 normalized but record a fatal write-failure issue.
                let mut updated = issues.clone();
                updated.push(IntakeIssue {
                    issue_id: format!("write-fail-{}", sanitize_id(path)),
                    severity: IssueSeverity::Fatal,
                    code: "WRITE_FAILED".to_owned(),
                    description: format!("Could not save normalized PDF '{path}': {e}"),
                    page_index: None,
                    suggested_action: Some(
                        "Check file permissions and available disk space.".to_owned(),
                    ),
                });
                return FileNormalizationResult {
                    path: path.to_owned(),
                    page_count,
                    issues: updated,
                    rotations_normalized: 0,
                };
            }
        }
    };

    FileNormalizationResult { path: path.to_owned(), page_count, issues, rotations_normalized }
}

/// Return `(1-based page_num, degrees)` for every page whose `/Rotate` value
/// is not a multiple of 360° (i.e. it is meaningfully rotated).
fn detect_rotations(doc: &Document, pages: &BTreeMap<u32, ObjectId>) -> Vec<(u32, i64)> {
    pages
        .iter()
        .filter_map(|(&page_num, &page_id)| {
            let deg = page_rotate_degrees(doc, page_id);
            if deg % 360 != 0 { Some((page_num, deg)) } else { None }
        })
        .collect()
}

/// Read the `/Rotate` integer from `page_id`'s dictionary, defaulting to 0.
fn page_rotate_degrees(doc: &Document, page_id: ObjectId) -> i64 {
    doc.objects
        .get(&page_id)
        .and_then(|obj| obj.as_dict().ok())
        .and_then(|dict| dict.get(b"Rotate").ok())
        .and_then(|obj| obj.as_i64().ok())
        .unwrap_or(0)
}

/// Set `/Rotate 0` on each of the given page object IDs in the document.
fn apply_normalization(doc: &mut Document, page_ids: &[ObjectId]) {
    for &page_id in page_ids {
        if let Some(Object::Dictionary(ref mut dict)) = doc.objects.get_mut(&page_id) {
            dict.set("Rotate", Object::Integer(0));
        }
    }
}

/// Build a stable, path-derived fragment for use in issue IDs.
fn sanitize_id(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' })
        .take(32)
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_contracts::intake::{IntakeBundle, IntakeFile, IntakeRole};
    use lopdf::{dictionary, Document, Object};

    // ── Test helpers ──────────────────────────────────────────────────────────

    /// Build a minimal `n_pages`-page PDF where the specified pages carry the
    /// given `/Rotate` values (all other pages default to 0°).
    ///
    /// `page_rotations` is a slice of `(1-based page_num, rotation_degrees)`.
    fn build_pdf_with_rotations(page_rotations: &[(u32, i64)]) -> Vec<u8> {
        let n_pages = page_rotations.iter().map(|(p, _)| *p).max().unwrap_or(3).max(3);
        let mut doc = Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let mut kids: Vec<Object> = Vec::new();
        for page_num in 1..=n_pages {
            let rotation =
                page_rotations.iter().find(|(p, _)| *p == page_num).map(|(_, r)| *r).unwrap_or(0);
            let mut page_dict = dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
                "MediaBox" => vec![0i64.into(), 0i64.into(), 612i64.into(), 792i64.into()],
            };
            if rotation != 0 {
                page_dict.set("Rotate", Object::Integer(rotation));
            }
            let page_id = doc.add_object(Object::Dictionary(page_dict));
            kids.push(Object::Reference(page_id));
        }
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => kids,
                "Count" => i64::from(n_pages),
            }),
        );
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut buf = Vec::new();
        doc.save_to(&mut buf).expect("write test PDF to buffer");
        buf
    }

    /// Write a test PDF with the given rotations to a temp file and return the path.
    fn write_rotated_pdf(stem: &str, rotations: &[(u32, i64)]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("conset-intake-tests");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join(format!("{stem}.pdf"));
        std::fs::write(&path, build_pdf_with_rotations(rotations)).expect("write PDF");
        path
    }

    // ── detect_rotations unit tests ───────────────────────────────────────────

    #[test]
    fn detect_rotation_all_zero_no_issues() {
        let bytes = build_pdf_with_rotations(&[]);
        let mut cursor = std::io::Cursor::new(bytes);
        let doc = Document::load_from(&mut cursor).unwrap();
        let pages = doc.get_pages();
        assert!(detect_rotations(&doc, &pages).is_empty());
    }

    #[test]
    fn detect_rotation_90_deg_found() {
        let bytes = build_pdf_with_rotations(&[(2, 90)]);
        let mut cursor = std::io::Cursor::new(bytes);
        let doc = Document::load_from(&mut cursor).unwrap();
        let pages = doc.get_pages();
        let rotated = detect_rotations(&doc, &pages);
        assert_eq!(rotated.len(), 1);
        assert_eq!(rotated[0], (2, 90));
    }

    #[test]
    fn detect_rotation_180_deg_found() {
        let bytes = build_pdf_with_rotations(&[(1, 180)]);
        let mut cursor = std::io::Cursor::new(bytes);
        let doc = Document::load_from(&mut cursor).unwrap();
        let pages = doc.get_pages();
        let rotated = detect_rotations(&doc, &pages);
        assert_eq!(rotated.len(), 1);
        assert_eq!(rotated[0], (1, 180));
    }

    #[test]
    fn detect_rotation_360_treated_as_zero() {
        // 360° is a multiple of 360 — should not be flagged as rotated.
        let bytes = build_pdf_with_rotations(&[(1, 360)]);
        let mut cursor = std::io::Cursor::new(bytes);
        let doc = Document::load_from(&mut cursor).unwrap();
        let pages = doc.get_pages();
        assert!(
            detect_rotations(&doc, &pages).is_empty(),
            "360° is equivalent to 0°; must not be flagged"
        );
    }

    // ── Stage0Normalizer integration tests ────────────────────────────────────

    #[test]
    fn normalize_corrects_rotated_page() {
        let path = write_rotated_pdf("nrm-rotate", &[(2, 90)]);
        let path_str = path.to_str().unwrap();
        let bundle = IntakeBundle {
            files: vec![IntakeFile { path: path_str.to_owned(), role: IntakeRole::OriginalSpec }],
            declared_order: None,
        };
        let result = Stage0Normalizer::normalize(&bundle, false);
        let f = &result.files[0];
        assert_eq!(f.rotations_normalized, 1);
        assert_eq!(f.issues.len(), 1);
        assert_eq!(f.issues[0].code, "ROTATED_PAGE");
        // Verify the rotation was cleared on disk.
        let doc = Document::load(path_str).unwrap();
        let pages = doc.get_pages();
        assert!(detect_rotations(&doc, &pages).is_empty(), "rotation must be cleared on disk");
    }

    #[test]
    fn normalize_page_count_unchanged() {
        let path = write_rotated_pdf("nrm-count", &[(1, 90), (3, 270)]);
        let path_str = path.to_str().unwrap();
        let bundle = IntakeBundle {
            files: vec![IntakeFile { path: path_str.to_owned(), role: IntakeRole::OriginalSpec }],
            declared_order: None,
        };
        let result = Stage0Normalizer::normalize(&bundle, false);
        let f = &result.files[0];
        // Helper builds max(rotations).max(3) pages → 3 pages here.
        assert_eq!(f.page_count, 3);
        assert_eq!(f.rotations_normalized, 2);
    }

    #[test]
    fn normalize_dry_run_leaves_file_unmodified() {
        let path = write_rotated_pdf("nrm-dry", &[(1, 90)]);
        let path_str = path.to_str().unwrap();
        let original_bytes = std::fs::read(path_str).unwrap();
        let bundle = IntakeBundle {
            files: vec![IntakeFile { path: path_str.to_owned(), role: IntakeRole::OriginalSpec }],
            declared_order: None,
        };
        let result = Stage0Normalizer::normalize(&bundle, true);
        let f = &result.files[0];
        // Issues are still reported in dry-run mode.
        assert_eq!(f.issues.len(), 1);
        assert_eq!(f.issues[0].code, "ROTATED_PAGE");
        assert_eq!(f.rotations_normalized, 0, "dry-run must not normalize any pages");
        // File on disk must be bit-for-bit identical.
        let after_bytes = std::fs::read(path_str).unwrap();
        assert_eq!(original_bytes, after_bytes, "dry-run must not modify the file");
    }
}
