//! Pattern database types and default-pattern loading.
//!
//! The default pattern database is embedded at compile time via
//! `include_str!()`. Call [`PatternDatabase::load_default`] to obtain a
//! parsed copy; it is cheap to clone and can be stored in a `OnceLock` if
//! needed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Opaque string key that identifies a pattern family, e.g.
/// `"footer-section-id"` or `"page-counter"`.
pub type FamilyId = String;

/// The page region that a pattern applies to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegionBand {
    /// Top ~15 % of the page (normalised Y < 0.15).
    Top,
    /// Bottom ~10 % of the page (normalised Y > 0.90).
    Bottom,
    /// Entire page — no geometric filter.
    Full,
}

/// Specification for a single pattern family.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternSpec {
    /// The compiled regex source string (Rust `regex` crate syntax).
    pub regex: String,
    /// Minimum hit-rate within a section required to accept the pattern match.
    pub confidence_threshold: f64,
    /// Page region the pattern is applied to.
    pub band: RegionBand,
    /// Canonical example strings that this pattern must match.
    #[serde(default)]
    pub examples: Vec<String>,
}

/// Versioned collection of [`PatternSpec`] entries.
///
/// The default database is embedded at compile time from
/// `src/patterns/default.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternDatabase {
    /// Semantic version string, e.g. `"1.0.0"`.
    pub version: String,
    /// Map from [`FamilyId`] to its [`PatternSpec`].
    pub patterns: HashMap<FamilyId, PatternSpec>,
}

impl PatternDatabase {
    /// Parse the embedded `default.json` and return it.
    ///
    /// The JSON is embedded at compile time via [`include_str!`], so network
    /// and filesystem IO never occur. Loading fails only if the embedded JSON
    /// is malformed; call this at startup and propagate the error to the
    /// caller.
    ///
    /// # Errors
    ///
    /// Returns a human-readable error string if the embedded JSON is invalid.
    pub fn load_default() -> Result<Self, String> {
        const EMBEDDED: &str = include_str!("default.json");
        serde_json::from_str(EMBEDDED)
            .map_err(|e| format!("pattern database 'default.json' is malformed: {e}"))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the embedded `default.json` deserializes without error
    /// and contains the expected pattern families.
    #[test]
    fn default_pattern_database_parses_successfully() {
        let db = PatternDatabase::load_default()
            .expect("default pattern database must parse without error");

        assert!(!db.version.is_empty(), "version must be non-empty");
        assert!(!db.patterns.is_empty(), "patterns map must be non-empty");

        assert!(
            db.patterns.contains_key("footer-section-id"),
            "must contain 'footer-section-id'"
        );
        assert!(
            db.patterns.contains_key("page-counter"),
            "must contain 'page-counter'"
        );
        assert!(
            db.patterns.contains_key("header-band"),
            "must contain 'header-band'"
        );

        let footer = &db.patterns["footer-section-id"];
        assert!(!footer.regex.is_empty(), "footer-section-id regex must be non-empty");
        assert!(
            footer.confidence_threshold > 0.0,
            "footer-section-id confidence_threshold must be > 0"
        );
        assert_eq!(footer.band, RegionBand::Bottom);

        let counter = &db.patterns["page-counter"];
        assert_eq!(counter.band, RegionBand::Bottom);

        let header = &db.patterns["header-band"];
        assert_eq!(header.band, RegionBand::Top);
    }
}
