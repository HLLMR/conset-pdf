//! Phase 6 PDF stitching engine — section page replacement via `lopdf`.
//!
//! [`PdfStitcher`] is the public entry point.  It accepts a [`StitchPlan`] and
//! returns a [`StitchResult`] or [`StitchError`].
//!
//! # Algorithm
//!
//! 1. Load *original* and *replacement* PDFs via [`lopdf::Document::load`].
//! 2. Look up the target section's page range in the [`SegmentIndex`].
//! 3. Renumber replacement objects to avoid object-ID collisions with the
//!    original, then copy all replacement objects into the original document.
//! 4. Rebuild the `/Pages` root `/Kids` array:
//!    `original[..del_start] + replacement pages + original[del_end+1..]`.
//! 5. Update `/Parent` references on replacement pages to point to the
//!    original's `/Pages` root.
//! 6. Remove the deleted section's `/Page` objects from the object store.
//! 7. Reroute any outline-item (`/Dest`) destinations that pointed to deleted
//!    pages so they target the first replacement page instead.
//! 8. Validate that object IDs for unchanged pages are still present.
//! 9. Write the output file (skipped on dry-run).
//!
//! # Invariants
//!
//! - **Read path:** PDFium (pdfium-render) — text/layout extraction only.
//! - **Write path:** `lopdf` — page-level operations only.
//! - These two libraries have non-overlapping responsibilities (MASTER_PLAN
//!   Non-Negotiable #23).

use std::collections::{HashMap, HashSet};

use conset_pdf_ir::{SegmentIndex, StitchError, StitchPlan, StitchResult};
use lopdf::{Document, Object, ObjectId};

/// Stateless PDF stitcher.
///
/// All logic lives in [`PdfStitcher::stitch`]; the struct is a zero-size
/// namespace.
pub struct PdfStitcher;

impl PdfStitcher {
    /// Replace a section's pages in an original PDF with pages from a
    /// regenerated replacement PDF.
    ///
    /// # Errors
    ///
    /// Returns [`StitchError`] if the original or replacement cannot be loaded,
    /// the section ID is not in the segment index, the page range is invalid,
    /// the PDF structure cannot be interpreted, or the output cannot be written.
    pub fn stitch(plan: &StitchPlan) -> Result<StitchResult, StitchError> {
        let mut doc = Document::load(&plan.original_path)
            .map_err(|e| StitchError::OriginalNotFound(format!("{}: {e}", plan.original_path)))?;
        let original_total = doc.get_pages().len();

        let (del_start, del_end) =
            resolve_section_range(&plan.segment_index, &plan.section_id, original_total)?;

        let orig_page_ids = sorted_page_ids(&doc);

        let (repl_page_ids, repl_max_id) =
            load_and_merge_replacement(&mut doc, &plan.replacement_path)?;
        if repl_max_id > doc.max_id {
            doc.max_id = repl_max_id;
        }

        // Snapshot content hashes for unchanged pages BEFORE splice mutates /Pages.
        let unchanged_ids: Vec<ObjectId> = orig_page_ids[..del_start]
            .iter()
            .chain(orig_page_ids[del_end + 1..].iter())
            .cloned()
            .collect();
        let before_hashes = snapshot_hashes(&doc, &unchanged_ids);

        splice_page_tree(&mut doc, &orig_page_ids, &repl_page_ids, del_start, del_end)?;

        let deleted_ids: HashSet<ObjectId> =
            orig_page_ids[del_start..=del_end].iter().cloned().collect();
        for &id in &orig_page_ids[del_start..=del_end] {
            doc.objects.remove(&id);
        }

        let bookmarks_rerouted =
            fixup_bookmarks(&mut doc, &deleted_ids, repl_page_ids.first().copied());

        let mut warnings =
            validate_unchanged_present(&orig_page_ids, del_start, del_end, &doc);
        validate_unchanged_content(&doc, &before_hashes, &mut warnings);

        if !plan.dry_run {
            doc.save(&plan.output_path)
                .map_err(|e| StitchError::WriteFailed(format!(
                    "could not write output PDF to '{}': {e} — \
                     check that the directory exists and the process has write permission",
                    plan.output_path
                )))?;
        }

        let pages_removed = del_end - del_start + 1;
        let pages_inserted = repl_page_ids.len();
        let new_total = original_total - pages_removed + pages_inserted;

        Ok(StitchResult {
            section_id: plan.section_id.clone(),
            pages_removed,
            pages_inserted,
            total_pages_before: original_total,
            total_pages_after: new_total,
            bookmarks_updated: bookmarks_rerouted > 0,
            warnings,
        })
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Resolve a section ID to `(start_page_idx, end_page_idx)` (both 0-based,
/// inclusive), validating that the range is within the document.
fn resolve_section_range(
    index: &SegmentIndex,
    section_id: &str,
    total_pages: usize,
) -> Result<(usize, usize), StitchError> {
    let entry = index
        .sections
        .iter()
        .find(|s| s.section_id == section_id)
        .ok_or_else(|| StitchError::SectionNotFound(section_id.to_owned()))?;

    let start = entry.start_page;
    let end = entry.end_page;

    if start > end {
        return Err(StitchError::PageRangeOutOfBounds(format!(
            "section '{section_id}' has start_page {start} > end_page {end}"
        )));
    }
    if end >= total_pages {
        return Err(StitchError::PageRangeOutOfBounds(format!(
            "section '{section_id}' end_page {end} >= document page count {total_pages}"
        )));
    }

    Ok((start, end))
}

/// Return page [`ObjectId`]s in ascending page-number order (0-based index →
/// lopdf 1-based page number).
fn sorted_page_ids(doc: &Document) -> Vec<ObjectId> {
    // BTreeMap keys are u32 page numbers; iterating values gives reading order.
    doc.get_pages().into_values().collect()
}

/// Load the replacement PDF, renumber its objects to avoid ID collision with
/// `doc`, copy all replacement objects into `doc`, and return the replacement's
/// ordered page object IDs plus its post-renumber `max_id`.
fn load_and_merge_replacement(
    doc: &mut Document,
    replacement_path: &str,
) -> Result<(Vec<ObjectId>, u32), StitchError> {
    let mut repl = Document::load(replacement_path)
        .map_err(|e| StitchError::ReplacementNotFound(format!("{replacement_path}: {e}")))?;

    repl.renumber_objects_with(doc.max_id + 1);
    let repl_page_ids = sorted_page_ids(&repl);
    let repl_max_id = repl.max_id;

    doc.objects.extend(repl.objects);

    Ok((repl_page_ids, repl_max_id))
}

/// Rebuild the `/Pages` root `/Kids` array and `/Count`, update `/Parent`
/// references on replacement pages, and leave unchanged pages untouched.
fn splice_page_tree(
    doc: &mut Document,
    orig_page_ids: &[ObjectId],
    repl_page_ids: &[ObjectId],
    del_start: usize,
    del_end: usize,
) -> Result<(), StitchError> {
    let pages_root_id = find_pages_root_id(doc)?;

    // Build the new /Kids: pages before section + replacement + pages after.
    let new_kids: Vec<Object> = orig_page_ids[..del_start]
        .iter()
        .chain(repl_page_ids.iter())
        .chain(orig_page_ids[del_end + 1..].iter())
        .map(|&id| Object::Reference(id))
        .collect();
    let new_count = new_kids.len();

    match doc.get_object_mut(pages_root_id) {
        Ok(obj) => {
            let Object::Dictionary(ref mut pages_dict) = obj else {
                return Err(StitchError::PdfStructure(
                    "/Pages root object is not a dictionary".to_owned(),
                ));
            };
            pages_dict.set("Kids", Object::Array(new_kids));
            pages_dict.set(
                "Count",
                Object::Integer(i64::try_from(new_count).unwrap_or(i64::MAX)),
            );
        }
        Err(e) => return Err(StitchError::PdfStructure(format!(
            "could not access /Pages root at object {pages_root_id:?} — \
             the PDF cross-reference table may be corrupt (lopdf: {e})"
        ))),
    }

    // Re-parent replacement pages to the original's /Pages root.
    for &page_id in repl_page_ids {
        if let Ok(obj) = doc.get_object_mut(page_id) {
            if let Object::Dictionary(ref mut d) = obj {
                d.set("Parent", Object::Reference(pages_root_id));
            }
        }
    }

    Ok(())
}

/// Return the [`ObjectId`] of the `/Pages` root from the document catalog.
fn find_pages_root_id(doc: &Document) -> Result<ObjectId, StitchError> {
    doc.catalog()
        .map_err(|e| StitchError::PdfStructure(format!(
            "could not read /Catalog from PDF — the file may be corrupt, \
             encrypted, or not a standard page-based document (lopdf: {e})"
        )))?
        .get(b"Pages")
        .and_then(|obj| obj.as_reference())
        .map_err(|e| StitchError::PdfStructure(format!(
            "/Catalog object has no valid /Pages reference — this may be a \
             portfolio, fillable form, or diagram rather than a specification \
             book (lopdf: {e})"
        )))
}

/// Reroute outline-item `/Dest` destinations that reference a deleted page to
/// point to `new_dest` (the first replacement page) instead.
///
/// Returns the number of bookmark destinations updated.
fn fixup_bookmarks(
    doc: &mut Document,
    deleted: &HashSet<ObjectId>,
    new_dest: Option<ObjectId>,
) -> u32 {
    let Some(new_page) = new_dest else { return 0 };

    // First pass: collect object IDs whose /Dest points to a deleted page.
    let ids_to_update: Vec<ObjectId> = doc
        .objects
        .iter()
        .filter_map(|(&id, obj)| {
            let Object::Dictionary(dict) = obj else { return None };
            let dest_page_ref = dict
                .get(b"Dest")
                .ok()
                .and_then(|d| d.as_array().ok())
                .and_then(|arr| arr.first())
                .and_then(|f| f.as_reference().ok())?;
            if deleted.contains(&dest_page_ref) { Some(id) } else { None }
        })
        .collect();

    let count = u32::try_from(ids_to_update.len()).unwrap_or(u32::MAX);

    // Second pass: update the collected objects (mutable borrow, no conflicts).
    for id in ids_to_update {
        if let Some(Object::Dictionary(ref mut dict)) = doc.objects.get_mut(&id) {
            dict.set(
                "Dest",
                Object::Array(vec![
                    Object::Reference(new_page),
                    Object::Name(b"Fit".to_vec()),
                ]),
            );
        }
    }

    count
}

/// Warn for any unchanged page object ID that is no longer present in the
/// document.  This guards against accidental removal of non-section pages.
fn validate_unchanged_present(
    original_page_ids: &[ObjectId],
    del_start: usize,
    del_end: usize,
    doc: &Document,
) -> Vec<String> {
    original_page_ids
        .iter()
        .enumerate()
        .filter(|(i, _)| *i < del_start || *i > del_end)
        .filter_map(|(i, id)| {
            if doc.objects.contains_key(id) {
                None
            } else {
                Some(format!(
                    "page at index {i} (object id {id:?}) unexpectedly absent from output"
                ))
            }
        })
        .collect()
}

/// Snapshot FNV-1a content hashes for the given object IDs.
///
/// Only IDs present in [`Document::objects`] are snapshotted; absent IDs are
/// silently skipped (they will be caught by [`validate_unchanged_present`]).
fn snapshot_hashes(doc: &Document, ids: &[ObjectId]) -> HashMap<ObjectId, u64> {
    ids.iter()
        .filter_map(|&id| doc.objects.get(&id).map(|obj| (id, fnv1a_object(obj))))
        .collect()
}

/// Check that unchanged page objects retain identical content after splicing.
///
/// Appends a warning string for every object whose FNV-1a hash differs from
/// the value captured by [`snapshot_hashes`] before `splice_page_tree` ran.
fn validate_unchanged_content(
    doc: &Document,
    before: &HashMap<ObjectId, u64>,
    warnings: &mut Vec<String>,
) {
    for (&id, &before_hash) in before {
        if let Some(obj) = doc.objects.get(&id) {
            let after_hash = fnv1a_object(obj);
            if after_hash != before_hash {
                warnings.push(format!(
                    "page object {id:?} content changed unexpectedly \
                     (before: {before_hash:#018x}, after: {after_hash:#018x})"
                ));
            }
        }
        // Missing objects are reported by validate_unchanged_present; skip here.
    }
}

/// Compute a FNV-1a-64 hash over the `Debug` representation of a lopdf `Object`.
///
/// Using `Debug` output gives a stable, dependency-free byte representation of
/// any `Object` variant without requiring lopdf's internal serializer.
fn fnv1a_object(obj: &Object) -> u64 {
    const BASIS: u64 = 14_695_981_039_346_656_037;
    const PRIME: u64 = 1_099_511_628_211;
    let repr = format!("{obj:?}");
    repr.bytes().fold(BASIS, |hash, byte| {
        hash.wrapping_mul(PRIME) ^ u64::from(byte)
    })
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use conset_pdf_ir::{ChromeMetadata, CoverageStats, SectionEntry, SegmentIndex};
    use lopdf::{dictionary, Document, Object};

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Build a minimal `n`-page PDF and return the bytes (not written to disk).
    fn build_test_pdf_bytes(n_pages: u32) -> Vec<u8> {
        let mut doc = Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let mut kids: Vec<Object> = Vec::new();
        for _ in 0..n_pages {
            let page_id = doc.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
                "MediaBox" => vec![0i64.into(), 0i64.into(), 612i64.into(), 792i64.into()],
            });
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

    /// Write test PDF bytes to a temp path and return the path.
    fn write_test_pdf(stem: &str, n_pages: u32) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("conset-stitch-tests");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join(format!("{stem}.pdf"));
        std::fs::write(&path, build_test_pdf_bytes(n_pages)).expect("write test PDF");
        path
    }

    /// Build a minimal [`SegmentIndex`] covering `section_id` → pages
    /// `[start..=end]` inside a `total_pages`-page document.
    fn make_index(
        section_id: &str,
        start: usize,
        end: usize,
        total_pages: usize,
    ) -> SegmentIndex {
        SegmentIndex {
            source_path: "test.pdf".to_owned(),
            chrome_metadata: ChromeMetadata::default(),
            sections: vec![SectionEntry {
                section_id: section_id.to_owned(),
                section_title: String::new(),
                start_page: start,
                end_page: end,
                page_count: end - start + 1,
                page_counter_detected: false,
                confidence: 1.0,
            }],
            coverage: CoverageStats {
                pages_total: total_pages,
                pages_tagged: total_pages,
                pages_missing_footer: 0,
                coverage_ratio: 1.0,
            },
        }
    }

    // ── resolve_section_range tests ───────────────────────────────────────────

    #[test]
    fn resolve_section_range_found() {
        let index = make_index("23 82 16", 2, 4, 10);
        let (start, end) = resolve_section_range(&index, "23 82 16", 10).unwrap();
        assert_eq!(start, 2);
        assert_eq!(end, 4);
    }

    #[test]
    fn resolve_section_range_not_found() {
        let index = make_index("23 82 16", 2, 4, 10);
        let err = resolve_section_range(&index, "99 99 99", 10).unwrap_err();
        assert!(matches!(err, StitchError::SectionNotFound(_)));
    }

    #[test]
    fn resolve_section_range_single_page() {
        let index = make_index("01 00 00", 5, 5, 10);
        let (start, end) = resolve_section_range(&index, "01 00 00", 10).unwrap();
        assert_eq!(start, 5);
        assert_eq!(end, 5);
    }

    #[test]
    fn resolve_section_range_out_of_bounds_returns_error() {
        let index = make_index("01 00 00", 8, 12, 10); // end_page 12 >= total 10
        let err = resolve_section_range(&index, "01 00 00", 10).unwrap_err();
        assert!(matches!(err, StitchError::PageRangeOutOfBounds(_)));
    }

    // ── dry-run stitch tests ──────────────────────────────────────────────────

    #[test]
    fn stitch_dry_run_writes_no_output_file() {
        let orig_path = write_test_pdf("dry-run-orig", 5);
        let repl_path = write_test_pdf("dry-run-repl", 2);
        let out_path = std::env::temp_dir()
            .join("conset-stitch-tests")
            .join("dry-run-output.pdf");
        // Ensure clean state.
        let _ = std::fs::remove_file(&out_path);

        let plan = StitchPlan {
            original_path: orig_path.to_string_lossy().into_owned(),
            section_id: "01 00 00".to_owned(),
            segment_index: make_index("01 00 00", 1, 2, 5),
            replacement_path: repl_path.to_string_lossy().into_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: true,
        };

        let result = PdfStitcher::stitch(&plan).expect("dry-run stitch must succeed");
        assert!(!out_path.exists(), "dry-run must not write output file");
        assert_eq!(result.pages_removed, 2);
        assert_eq!(result.pages_inserted, 2);
        assert_eq!(result.total_pages_before, 5);
        assert_eq!(result.total_pages_after, 5); // 5 - 2 + 2
    }

    #[test]
    fn stitch_page_count_correct() {
        // 6-page original; replace pages 0..=1 (2 pages) with 3-page replacement.
        let orig_path = write_test_pdf("count-orig", 6);
        let repl_path = write_test_pdf("count-repl", 3);
        let out_path = std::env::temp_dir()
            .join("conset-stitch-tests")
            .join("count-output.pdf");
        let _ = std::fs::remove_file(&out_path);

        let plan = StitchPlan {
            original_path: orig_path.to_string_lossy().into_owned(),
            section_id: "01 00 00".to_owned(),
            segment_index: make_index("01 00 00", 0, 1, 6),
            replacement_path: repl_path.to_string_lossy().into_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: false,
        };

        let result = PdfStitcher::stitch(&plan).expect("stitch must succeed");
        assert_eq!(result.pages_removed, 2);
        assert_eq!(result.pages_inserted, 3);
        assert_eq!(result.total_pages_before, 6);
        assert_eq!(result.total_pages_after, 7); // 6 - 2 + 3

        assert!(out_path.exists(), "output PDF must be written");

        // Verify the output PDF has the correct page count via lopdf.
        let out_doc =
            Document::load(&out_path).expect("output must be a valid loadable PDF");
        assert_eq!(
            out_doc.get_pages().len(),
            7,
            "output PDF must have exactly 7 pages"
        );

        // Verify output starts with PDF header.
        let bytes = std::fs::read(&out_path).expect("read output PDF");
        assert!(bytes.starts_with(b"%PDF"), "output must start with %PDF");
    }

    #[test]
    fn stitch_replace_last_section() {
        // Replace the last 2 pages (indices 3..=4) with 1-page replacement.
        let orig_path = write_test_pdf("last-orig", 5);
        let repl_path = write_test_pdf("last-repl", 1);
        let out_path = std::env::temp_dir()
            .join("conset-stitch-tests")
            .join("last-output.pdf");
        let _ = std::fs::remove_file(&out_path);

        let plan = StitchPlan {
            original_path: orig_path.to_string_lossy().into_owned(),
            section_id: "99 99 99".to_owned(),
            segment_index: make_index("99 99 99", 3, 4, 5),
            replacement_path: repl_path.to_string_lossy().into_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: false,
        };

        let result = PdfStitcher::stitch(&plan).expect("stitch last section must succeed");
        assert_eq!(result.total_pages_after, 4); // 5 - 2 + 1
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn stitch_section_not_found_returns_error() {
        let orig_path = write_test_pdf("nf-orig", 3);
        let repl_path = write_test_pdf("nf-repl", 1);
        let out_path = std::env::temp_dir().join("conset-stitch-tests").join("nf-out.pdf");

        let plan = StitchPlan {
            original_path: orig_path.to_string_lossy().into_owned(),
            section_id: "55 55 55".to_owned(), // not in index
            segment_index: make_index("23 82 16", 0, 0, 3),
            replacement_path: repl_path.to_string_lossy().into_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: true,
        };

        let err = PdfStitcher::stitch(&plan).unwrap_err();
        assert!(matches!(err, StitchError::SectionNotFound(_)));
    }

    #[test]
    fn stitch_missing_original_returns_error() {
        let repl_path = write_test_pdf("mo-repl", 1);
        let out_path = std::env::temp_dir().join("conset-stitch-tests").join("mo-out.pdf");

        let plan = StitchPlan {
            original_path: "/nonexistent/original.pdf".to_owned(),
            section_id: "01 00 00".to_owned(),
            segment_index: make_index("01 00 00", 0, 0, 1),
            replacement_path: repl_path.to_string_lossy().into_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: false,
        };

        let err = PdfStitcher::stitch(&plan).unwrap_err();
        assert!(matches!(err, StitchError::OriginalNotFound(_)));
    }

    #[test]
    fn stitch_missing_replacement_returns_error() {
        let orig_path = write_test_pdf("mr-orig", 3);
        let out_path = std::env::temp_dir().join("conset-stitch-tests").join("mr-out.pdf");

        let plan = StitchPlan {
            original_path: orig_path.to_string_lossy().into_owned(),
            section_id: "01 00 00".to_owned(),
            segment_index: make_index("01 00 00", 0, 0, 3),
            replacement_path: "/nonexistent/replacement.pdf".to_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: false,
        };

        let err = PdfStitcher::stitch(&plan).unwrap_err();
        assert!(matches!(err, StitchError::ReplacementNotFound(_)));
    }

    // ── 8.1.A: malformed-input error-path tests ───────────────────────────────

    /// Corrupt bytes in the *original* must return `OriginalNotFound` rather
    /// than panicking.
    #[test]
    fn stitch_corrupt_original_returns_error_not_panic() {
        let dir = std::env::temp_dir().join("conset-stitch-tests");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let corrupt_path = dir.join("corrupt-orig.pdf");
        // Write clearly invalid PDF bytes — lopdf must return Err, not panic.
        std::fs::write(&corrupt_path, b"NOT A PDF\x00\xFF garbage").expect("write corrupt file");

        let repl_path = write_test_pdf("corrupt-orig-repl", 1);
        let out_path = dir.join("corrupt-orig-out.pdf");

        let plan = StitchPlan {
            original_path: corrupt_path.to_string_lossy().into_owned(),
            section_id: "01 00 00".to_owned(),
            segment_index: make_index("01 00 00", 0, 0, 1),
            replacement_path: repl_path.to_string_lossy().into_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: false,
        };

        let err = PdfStitcher::stitch(&plan).unwrap_err();
        assert!(
            matches!(err, StitchError::OriginalNotFound(_)),
            "corrupt original must return OriginalNotFound, got: {err:?}"
        );
    }

    /// A section range that exceeds the document page count must return
    /// `PageRangeOutOfBounds` rather than panicking with an index out-of-bounds.
    #[test]
    fn stitch_page_range_exceeds_document_returns_error_not_panic() {
        let orig_path = write_test_pdf("oob-orig", 2); // 2-page PDF
        let repl_path = write_test_pdf("oob-repl", 1);
        let out_path = std::env::temp_dir().join("conset-stitch-tests").join("oob-out.pdf");

        // Claim section spans pages 0–4 on a 2-page document.
        let plan = StitchPlan {
            original_path: orig_path.to_string_lossy().into_owned(),
            section_id: "01 00 00".to_owned(),
            segment_index: make_index("01 00 00", 0, 4, 5), // end_page=4 >= total=2
            replacement_path: repl_path.to_string_lossy().into_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: true,
        };

        let err = PdfStitcher::stitch(&plan).unwrap_err();
        assert!(
            matches!(err, StitchError::PageRangeOutOfBounds(_)),
            "out-of-bounds range must return PageRangeOutOfBounds, got: {err:?}"
        );
    }

    /// A corrupt *replacement* PDF (valid original, corrupt replacement) must
    /// return `ReplacementNotFound` / a lopdf error — not a panic.
    #[test]
    fn stitch_corrupt_replacement_returns_error_not_panic() {
        let orig_path = write_test_pdf("corrupt-repl-orig", 3);
        let dir = std::env::temp_dir().join("conset-stitch-tests");
        let corrupt_repl = dir.join("corrupt-repl.pdf");
        std::fs::write(&corrupt_repl, b"%PDF-1.4 truncated\x00").expect("write corrupt file");
        let out_path = dir.join("corrupt-repl-out.pdf");

        let plan = StitchPlan {
            original_path: orig_path.to_string_lossy().into_owned(),
            section_id: "01 00 00".to_owned(),
            segment_index: make_index("01 00 00", 0, 2, 3),
            replacement_path: corrupt_repl.to_string_lossy().into_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: false,
        };

        let err = PdfStitcher::stitch(&plan).unwrap_err();
        // Corrupt replacement may surface as ReplacementNotFound or PdfStructure
        // depending on how lopdf categorises the parse failure.
        assert!(
            matches!(err, StitchError::ReplacementNotFound(_) | StitchError::PdfStructure(_)),
            "corrupt replacement must return a stitch error, got: {err:?}"
        );
    }

    // ── 8.3.E — Unchanged-page content-hash tests ─────────────────────────────

    /// After a successful stitch, unchanged page objects must have identical
    /// in-memory content before and after `splice_page_tree` (no hash mismatches).
    #[test]
    fn stitch_unchanged_pages_content_hash_unchanged() {
        let orig_path = write_test_pdf("hash-orig", 5);
        let repl_path = write_test_pdf("hash-repl", 2);
        let out_path =
            std::env::temp_dir().join("conset-stitch-tests").join("hash-out.pdf");
        let _ = std::fs::remove_file(&out_path);

        // Replace the middle section (pages 1–2 in a 5-page doc: 0-indexed).
        let plan = StitchPlan {
            original_path: orig_path.to_string_lossy().into_owned(),
            section_id: "01 00 00".to_owned(),
            segment_index: make_index("01 00 00", 1, 2, 5),
            replacement_path: repl_path.to_string_lossy().into_owned(),
            output_path: out_path.to_string_lossy().into_owned(),
            dry_run: false,
        };

        let result = PdfStitcher::stitch(&plan).expect("stitch must succeed");
        assert!(
            result.warnings.is_empty(),
            "no content-hash warnings expected for a clean stitch, got: {:?}",
            result.warnings
        );
    }

    // ── 8.3.F — Multi-section page-growth regression test ────────────────────

    /// Build a `SegmentIndex` with three explicitly specified sections covering
    /// a `total_pages`-page document.
    fn make_three_section_index(total_pages: usize) -> SegmentIndex {
        SegmentIndex {
            source_path: "test.pdf".to_owned(),
            chrome_metadata: ChromeMetadata::default(),
            sections: vec![
                SectionEntry {
                    section_id: "sec-A".to_owned(),
                    section_title: String::new(),
                    start_page: 0,
                    end_page: 2,
                    page_count: 3,
                    page_counter_detected: false,
                    confidence: 1.0,
                },
                SectionEntry {
                    section_id: "sec-B".to_owned(),
                    section_title: String::new(),
                    start_page: 3,
                    end_page: 5,
                    page_count: 3,
                    page_counter_detected: false,
                    confidence: 1.0,
                },
                SectionEntry {
                    section_id: "sec-C".to_owned(),
                    section_title: String::new(),
                    start_page: 6,
                    end_page: 8,
                    page_count: 3,
                    page_counter_detected: false,
                    confidence: 1.0,
                },
            ],
            coverage: CoverageStats {
                pages_total: total_pages,
                pages_tagged: total_pages,
                pages_missing_footer: 0,
                coverage_ratio: 1.0,
            },
        }
    }

    /// Stitch two sections (C then A) with page-count growth to verify that the
    /// last-to-first ordering keeps the middle section (B) intact.
    ///
    /// Original: 9 pages (A=0–2, B=3–5, C=6–8).
    /// Step 1: replace C (3 pages) with 4 pages → total = 10, A/B unchanged.
    /// Step 2: replace A (3 pages) with 5 pages → total = 12, B unchanged.
    ///
    /// After both stitches:
    ///   - total pages = 5 (A) + 3 (B) + 4 (C) = 12
    ///   - all original B page object IDs are still present in the output
    ///   - no content-hash warnings (B's page objects are byte-identical)
    #[test]
    fn stitch_two_sections_with_page_growth_preserves_middle_section() {
        let dir = std::env::temp_dir().join("conset-stitch-tests");
        std::fs::create_dir_all(&dir).expect("create temp dir");

        // Build original 9-page doc and write to disk.
        let orig_path = dir.join("pgrowth-orig.pdf");
        std::fs::write(&orig_path, build_test_pdf_bytes(9)).expect("write orig PDF");

        // Replacement for C: 4 pages (net +1).
        let repl_c_path = write_test_pdf("pgrowth-repl-c", 4);
        // Replacement for A: 5 pages (net +2).
        let repl_a_path = write_test_pdf("pgrowth-repl-a", 5);

        // Capture B page object IDs from the original before any stitch.
        let orig_doc = Document::load(&orig_path).expect("load original doc");
        let orig_page_ids: Vec<ObjectId> = orig_doc.get_pages().into_values().collect();
        // B pages are at 0-indexed positions 3, 4, 5.
        let b_page_ids: Vec<ObjectId> =
            orig_page_ids[3..=5].iter().cloned().collect();

        let after_c_path = dir.join("pgrowth-after-c.pdf");
        let after_a_path = dir.join("pgrowth-after-a.pdf");
        let _ = std::fs::remove_file(&after_c_path);
        let _ = std::fs::remove_file(&after_a_path);

        // Step 1: replace section C (pages 6–8) last → first ordering.
        let result_c = PdfStitcher::stitch(&StitchPlan {
            original_path: orig_path.to_string_lossy().into_owned(),
            section_id: "sec-C".to_owned(),
            segment_index: make_three_section_index(9),
            replacement_path: repl_c_path.to_string_lossy().into_owned(),
            output_path: after_c_path.to_string_lossy().into_owned(),
            dry_run: false,
        })
        .expect("stitch C must succeed");
        assert_eq!(result_c.total_pages_after, 10, "after C: 9 - 3 + 4 = 10");
        assert!(result_c.warnings.is_empty(), "stitch C must have no warnings");

        // Build an updated SegmentIndex for the intermediate doc: A is still at
        // pages 0–2; B is still at 3–5 (C was after them, didn't shift A/B).
        let intermediate_index = {
            let mut idx = make_three_section_index(10);
            // A and B are unchanged in position; C now occupies 6–9 (4 pages).
            idx.sections[2].end_page = 9;
            idx.sections[2].page_count = 4;
            idx.coverage.pages_total = 10;
            idx.coverage.pages_tagged = 10;
            idx
        };

        // Step 2: replace section A (pages 0–2) in the intermediate doc.
        let result_a = PdfStitcher::stitch(&StitchPlan {
            original_path: after_c_path.to_string_lossy().into_owned(),
            section_id: "sec-A".to_owned(),
            segment_index: intermediate_index,
            replacement_path: repl_a_path.to_string_lossy().into_owned(),
            output_path: after_a_path.to_string_lossy().into_owned(),
            dry_run: false,
        })
        .expect("stitch A must succeed");

        // Total pages: A(5) + B(3) + C(4) = 12.
        assert_eq!(result_a.total_pages_after, 12, "final doc must have 12 pages");
        assert!(result_a.warnings.is_empty(), "stitch A must have no warnings");

        // Verify all original B page object IDs are still in the final document.
        let final_doc = Document::load(&after_a_path).expect("load final doc");
        for b_id in &b_page_ids {
            assert!(
                final_doc.objects.contains_key(b_id),
                "original B page object {b_id:?} must be present in final doc"
            );
        }
    }
}
