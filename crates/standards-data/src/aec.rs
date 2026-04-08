//! AEC discipline classification for drawing sheet IDs.
//!
//! Implements the 5-step algorithm from `docs/DRAWINGS_CLASSIFICATION.md`.

/// Version string for the embedded AEC data tables.
#[must_use]
pub fn data_version() -> &'static str {
    "1.0.0"
}

/// Result of classifying a drawing sheet number.
///
/// `canonical4` is the 4-character discipline code used throughout the codebase
/// (e.g. `"MECH"`, `"FIRP"`, `"CIVL"`, `"UNKN"`).
///
/// `sort_order` places disciplines in the NCS Display Order sequence (lower =
/// earlier in the group).  Unknown / unclassified sheets sort last (999).
#[derive(Debug, Clone, PartialEq)]
pub struct ClassifyResult {
    /// 4-character canonical discipline code (e.g. `"MECH"`).
    pub canonical4: &'static str,
    /// Human-readable display name (e.g. `"Mechanical"`).
    pub display_name: &'static str,
    /// NCS sort order.
    pub sort_order: u32,
    /// Classification confidence ∈ [0.0, 1.0].
    pub confidence: f64,
}

/// Classify a drawing sheet by its sheet number and optional title.
///
/// Implements the 5-step algorithm from `docs/DRAWINGS_CLASSIFICATION.md`:
///
/// 1. Extract the leading letter(s) from `sheet_number`.
/// 2. Match against the 13 UDS single-letter table.
/// 3. Match against the 10 multi-letter alias table.
/// 4. Disambiguate `C` using title keywords (CONTROLS vs CIVIL).
/// 5. Return `UNKN` if nothing matched.
///
/// # Arguments
///
/// * `sheet_number` – The raw sheet number string (e.g. `"M-201"`, `"FP-101"`).
/// * `sheet_title`  – Optional title from the title block used for disambiguation.
#[must_use]
pub fn classify_sheet(sheet_number: &str, sheet_title: Option<&str>) -> ClassifyResult {
    // ── Step 1: extract designator ─────────────────────────────────────────
    // Strip everything that is not ASCII alpha; collect the leading run of
    // letters only, then uppercase.
    let designator: String = sheet_number
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_uppercase();

    // ── Step 2: C disambiguation (applied before the UDS table so the result
    //   is available when the UDS table hits C) ────────────────────────────
    if designator == "C" {
        return disambiguate_c(sheet_title);
    }

    // ── Step 3: UDS single-letter table ───────────────────────────────────
    if let Some(r) = uds_single_letter(&designator) {
        return r;
    }

    // ── Step 4: multi-letter alias table ──────────────────────────────────
    if let Some(r) = multi_letter_alias(&designator) {
        return r;
    }

    // ── Step 5: UNKN fallback ─────────────────────────────────────────────
    ClassifyResult {
        canonical4: "UNKN",
        display_name: "Unknown",
        sort_order: 999,
        confidence: 0.0,
    }
}

// ── Internal tables ───────────────────────────────────────────────────────────

/// 13-entry UDS single-letter table (excludes `C`, which is dispatched to
/// `disambiguate_c` before this function is called).
fn uds_single_letter(designator: &str) -> Option<ClassifyResult> {
    match designator {
        "G" => Some(ClassifyResult {
            canonical4: "GENR",
            display_name: "General",
            sort_order: 10,
            confidence: 1.0,
        }),
        "V" => Some(ClassifyResult {
            canonical4: "SURV",
            display_name: "Survey",
            sort_order: 20,
            confidence: 1.0,
        }),
        "D" => Some(ClassifyResult {
            canonical4: "PROC",
            display_name: "Process",
            sort_order: 30,
            confidence: 1.0,
        }),
        "L" => Some(ClassifyResult {
            canonical4: "LAND",
            display_name: "Landscape",
            sort_order: 40,
            confidence: 1.0,
        }),
        "A" => Some(ClassifyResult {
            canonical4: "ARCH",
            display_name: "Architectural",
            sort_order: 50,
            confidence: 1.0,
        }),
        "I" => Some(ClassifyResult {
            canonical4: "INTR",
            display_name: "Interiors",
            sort_order: 60,
            confidence: 1.0,
        }),
        "S" => Some(ClassifyResult {
            canonical4: "STRU",
            display_name: "Structural",
            sort_order: 65,
            confidence: 1.0,
        }),
        "M" => Some(ClassifyResult {
            canonical4: "MECH",
            display_name: "Mechanical",
            sort_order: 70,
            confidence: 1.0,
        }),
        "P" => Some(ClassifyResult {
            canonical4: "PLUM",
            display_name: "Plumbing",
            sort_order: 80,
            confidence: 1.0,
        }),
        "E" => Some(ClassifyResult {
            canonical4: "ELEC",
            display_name: "Electrical",
            sort_order: 90,
            confidence: 1.0,
        }),
        "F" => Some(ClassifyResult {
            canonical4: "FIRP",
            display_name: "Fire Protection",
            sort_order: 75,
            confidence: 1.0,
        }),
        "T" => Some(ClassifyResult {
            canonical4: "TECH",
            display_name: "Technology",
            sort_order: 100,
            confidence: 1.0,
        }),
        _ => None,
    }
}

/// 10-entry multi-letter alias table.
fn multi_letter_alias(designator: &str) -> Option<ClassifyResult> {
    match designator {
        "FP" | "FA" => Some(ClassifyResult {
            canonical4: "FIRP",
            display_name: "Fire Protection",
            sort_order: 75,
            confidence: 0.95,
        }),
        "DDC" | "ATC" => Some(ClassifyResult {
            canonical4: "CTRL",
            display_name: "Controls",
            sort_order: 95,
            confidence: 0.95,
        }),
        "SEC" | "AV" | "IT" => Some(ClassifyResult {
            canonical4: "TECH",
            display_name: "Technology",
            sort_order: 100,
            confidence: 0.90,
        }),
        "SV" => Some(ClassifyResult {
            canonical4: "SURV",
            display_name: "Survey",
            sort_order: 20,
            confidence: 0.85,
        }),
        "DM" => Some(ClassifyResult {
            canonical4: "DEMO",
            display_name: "Demolition",
            sort_order: 35,
            confidence: 0.85,
        }),
        "EX" => Some(ClassifyResult {
            canonical4: "UNKN",
            display_name: "Unknown",
            sort_order: 999,
            confidence: 0.80,
        }),
        _ => None,
    }
}

/// Step-4 disambiguation for the `C` designator.
///
/// CONTROLS keywords → `CTRL/0.85`; CIVIL keywords → `CIVL/0.85`;
/// no keywords or no title → `CIVL/0.72` (civil is the common case in AEC).
fn disambiguate_c(sheet_title: Option<&str>) -> ClassifyResult {
    const CONTROLS_KEYWORDS: &[&str] = &[
        "CONTROL",
        "CONTROLS",
        "DDC",
        "BAS",
        "BMS",
        "HVAC CONTROL",
        "BUILDING AUTOMATION",
        "TEMPERATURE CONTROL",
    ];
    const CIVIL_KEYWORDS: &[&str] = &[
        "CIVIL",
        "SITE",
        "GRADING",
        "DRAINAGE",
        "UTILITY",
        "PAVING",
        "ROAD",
    ];

    if let Some(title) = sheet_title {
        let upper = title.to_uppercase();

        if CONTROLS_KEYWORDS.iter().any(|kw| upper.contains(kw)) {
            return ClassifyResult {
                canonical4: "CTRL",
                display_name: "Controls",
                sort_order: 95,
                confidence: 0.85,
            };
        }

        if CIVIL_KEYWORDS.iter().any(|kw| upper.contains(kw)) {
            return ClassifyResult {
                canonical4: "CIVL",
                display_name: "Civil",
                sort_order: 25,
                confidence: 0.85,
            };
        }
    }

    // Default: C → Civil (most common in AEC drawings)
    ClassifyResult {
        canonical4: "CIVL",
        display_name: "Civil",
        sort_order: 25,
        confidence: 0.72,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── UDS single-letter table ───────────────────────────────────────────

    #[test]
    fn uds_g_general() {
        let r = classify_sheet("G-001", None);
        assert_eq!(r.canonical4, "GENR");
        assert!((r.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uds_v_survey() {
        let r = classify_sheet("V-100", None);
        assert_eq!(r.canonical4, "SURV");
        assert_eq!(r.sort_order, 20);
    }

    #[test]
    fn uds_d_process() {
        let r = classify_sheet("D-500", None);
        assert_eq!(r.canonical4, "PROC");
    }

    #[test]
    fn uds_l_landscape() {
        let r = classify_sheet("L-201", None);
        assert_eq!(r.canonical4, "LAND");
    }

    #[test]
    fn uds_a_architectural() {
        let r = classify_sheet("A-101", None);
        assert_eq!(r.canonical4, "ARCH");
        assert_eq!(r.sort_order, 50);
    }

    #[test]
    fn uds_i_interiors() {
        let r = classify_sheet("I-101", None);
        assert_eq!(r.canonical4, "INTR");
    }

    #[test]
    fn uds_s_structural() {
        let r = classify_sheet("S-101", None);
        assert_eq!(r.canonical4, "STRU");
    }

    #[test]
    fn uds_m_mechanical() {
        let r = classify_sheet("M-201", None);
        assert_eq!(r.canonical4, "MECH");
        assert_eq!(r.sort_order, 70);
        assert!((r.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uds_p_plumbing() {
        let r = classify_sheet("P-101", None);
        assert_eq!(r.canonical4, "PLUM");
    }

    #[test]
    fn uds_e_electrical() {
        let r = classify_sheet("E-101", None);
        assert_eq!(r.canonical4, "ELEC");
        assert_eq!(r.sort_order, 90);
    }

    #[test]
    fn uds_f_fire_protection() {
        let r = classify_sheet("F-101", None);
        assert_eq!(r.canonical4, "FIRP");
        assert_eq!(r.sort_order, 75);
    }

    #[test]
    fn uds_t_technology() {
        let r = classify_sheet("T-101", None);
        assert_eq!(r.canonical4, "TECH");
        assert_eq!(r.sort_order, 100);
    }

    // ── Multi-letter alias table ──────────────────────────────────────────

    #[test]
    fn alias_fp_fire_protection() {
        let r = classify_sheet("FP-101", None);
        assert_eq!(r.canonical4, "FIRP");
        assert!((r.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn alias_fa_fire_protection() {
        let r = classify_sheet("FA-101", None);
        assert_eq!(r.canonical4, "FIRP");
    }

    #[test]
    fn alias_ddc_controls() {
        let r = classify_sheet("DDC-101", None);
        assert_eq!(r.canonical4, "CTRL");
        assert!((r.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn alias_atc_controls() {
        let r = classify_sheet("ATC-201", None);
        assert_eq!(r.canonical4, "CTRL");
    }

    #[test]
    fn alias_sec_technology() {
        let r = classify_sheet("SEC-101", None);
        assert_eq!(r.canonical4, "TECH");
        assert!((r.confidence - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn alias_av_technology() {
        let r = classify_sheet("AV-101", None);
        assert_eq!(r.canonical4, "TECH");
    }

    #[test]
    fn alias_it_technology() {
        let r = classify_sheet("IT-101", None);
        assert_eq!(r.canonical4, "TECH");
    }

    #[test]
    fn alias_sv_survey() {
        let r = classify_sheet("SV-101", None);
        assert_eq!(r.canonical4, "SURV");
        assert!((r.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn alias_dm_demolition() {
        let r = classify_sheet("DM-101", None);
        assert_eq!(r.canonical4, "DEMO");
        assert!((r.confidence - 0.85).abs() < f64::EPSILON);
    }

    // ── C disambiguation ─────────────────────────────────────────────────

    #[test]
    fn c_no_title_defaults_to_civil() {
        let r = classify_sheet("C-101", None);
        assert_eq!(r.canonical4, "CIVL");
        assert!((r.confidence - 0.72).abs() < f64::EPSILON);
    }

    #[test]
    fn c_civil_keyword_in_title() {
        let r = classify_sheet("C-101", Some("SITE GRADING PLAN"));
        assert_eq!(r.canonical4, "CIVL");
        assert!((r.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn c_controls_keyword_in_title() {
        let r = classify_sheet("C-101", Some("HVAC CONTROL SCHEMATIC"));
        assert_eq!(r.canonical4, "CTRL");
        assert!((r.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn c_bms_keyword_in_title() {
        let r = classify_sheet("C-201", Some("BMS SEQUENCE OF OPERATIONS"));
        assert_eq!(r.canonical4, "CTRL");
    }

    #[test]
    fn c_utility_keyword_falls_through_to_civil() {
        let r = classify_sheet("C-301", Some("UTILITY PLAN"));
        assert_eq!(r.canonical4, "CIVL");
        assert!((r.confidence - 0.85).abs() < f64::EPSILON);
    }

    // ── Number format variants ────────────────────────────────────────────

    #[test]
    fn sheet_id_with_leading_zeros() {
        let r = classify_sheet("M001", None);
        assert_eq!(r.canonical4, "MECH");
    }

    #[test]
    fn sheet_id_lowercase_prefix() {
        let r = classify_sheet("fp-101", None);
        assert_eq!(r.canonical4, "FIRP");
    }

    #[test]
    fn sheet_id_mixed_case_prefix() {
        let r = classify_sheet("Fp-101", None);
        assert_eq!(r.canonical4, "FIRP");
    }

    // ── UNKN fallback ─────────────────────────────────────────────────────

    #[test]
    fn unknown_designator_returns_unkn() {
        let r = classify_sheet("X-101", None);
        assert_eq!(r.canonical4, "UNKN");
        assert_eq!(r.sort_order, 999);
    }

    #[test]
    fn empty_sheet_number_returns_unkn() {
        let r = classify_sheet("", None);
        assert_eq!(r.canonical4, "UNKN");
    }

    #[test]
    fn numeric_only_sheet_number_returns_unkn() {
        let r = classify_sheet("101", None);
        assert_eq!(r.canonical4, "UNKN");
    }

    // ── data_version check ────────────────────────────────────────────────

    #[test]
    fn data_version_is_not_stub() {
        assert_ne!(data_version(), "stub");
        assert_eq!(data_version(), "1.0.0");
    }
}
