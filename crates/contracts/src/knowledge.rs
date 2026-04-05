//! Knowledge base contract types — Phase 0.5 contract-shaping.
//!
//! Covers the three canonical knowledge-layer record types:
//!
//! - **Normalization records** — link raw extracted values to canonical
//!   standard references (Band 3, G-038).
//! - **Entity records** — resolved named entities such as manufacturers,
//!   products, and spec sections (Band 3, G-039).
//! - **Index records** — searchable surface entries over document sections
//!   (Band 3, G-040).
//!
//! **CONTRACT-ONLY** — all runtime indexing deferred to Band 3.

use serde::{Deserialize, Serialize};

/// Schema version for knowledge contract types.
pub const KNOWLEDGE_SCHEMA_VERSION: &str = "0.5.0";

// ── Normalization record ───────────────────────────────────────────────────

/// Links a raw extracted string value to a canonical standard reference.
///
/// Normalization records are produced by the Band 3 standards-matching pass
/// and consumed by the export layer to emit clean, consistent field values.
///
/// **CONTRACT-ONLY** — normalization runtime deferred to Band 3 (G-038).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationRecord {
    pub schema_version: String,
    pub record_id: String,
    /// The raw string as extracted from the PDF.
    pub raw_value: String,
    /// Normalized canonical form, or `None` if no match was resolved.
    pub canonical_value: Option<String>,
    /// Standard reference identifier, e.g. `"MasterFormat:23 82 16"`.
    pub standard_ref: Option<String>,
    /// Confidence in the normalization match (0.0 – 1.0).
    pub confidence: Option<f32>,
    /// Source label: `"deterministic"`, `"micro_ml"`, `"llm"`, `"human"`.
    pub normalization_source: Option<String>,
    pub document_id: Option<String>,
    pub page_index: Option<usize>,
}

// ── Entity record ─────────────────────────────────────────────────────────

/// A resolved named entity (manufacturer, product, standard, person).
///
/// Entity records aggregate all surface mentions of a named thing across
/// the full document set into a single canonical record.
///
/// **CONTRACT-ONLY** — entity resolution runtime deferred to Band 3 (G-039).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRecord {
    pub schema_version: String,
    pub entity_id: String,
    /// Class label: `"manufacturer"`, `"product"`, `"standard"`, `"person"`,
    /// `"organization"`, or `"unknown"`.
    pub entity_type: String,
    /// Preferred display form of the entity name.
    pub canonical_name: Option<String>,
    /// All known alternative names / abbreviations for this entity.
    pub aliases: Vec<String>,
    /// Intake bundle IDs that mention this entity.
    pub source_bundle_ids: Vec<String>,
    /// Confidence in the entity resolution (0.0 – 1.0).
    pub confidence: Option<f32>,
    /// Standard reference if the entity maps to a known standard ID.
    pub standard_ref: Option<String>,
}

// ── Index record ──────────────────────────────────────────────────────────

/// A searchable surface entry for a document section or page element.
///
/// Index records are the unit of the full-text and semantic index: each
/// record covers one logical unit of content (a section, a table, a detail)
/// and carries back-references to entity and standard records.
///
/// **CONTRACT-ONLY** — indexing runtime deferred to Band 3 (G-040).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRecord {
    pub schema_version: String,
    pub index_id: String,
    /// Intake bundle ID of the source document.
    pub bundle_id: String,
    pub page_index: Option<usize>,
    /// Section identifier (e.g. `"23 82 16"`, `"S1.0"`, `"footer"`).
    pub section_id: Option<String>,
    /// Short human-readable summary of the indexed content.
    pub content_summary: Option<String>,
    /// `entity_id` values for entities mentioned in this section.
    pub entity_refs: Vec<String>,
    /// Standard reference identifiers (e.g. `"MasterFormat:23 82 16"`).
    pub standard_refs: Vec<String>,
    pub indexed_at_utc: Option<String>,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_record_round_trips_via_serde() {
        let r = NormalizationRecord {
            schema_version: KNOWLEDGE_SCHEMA_VERSION.to_owned(),
            record_id: "nr-001".to_owned(),
            raw_value: "FCU-1A".to_owned(),
            canonical_value: Some("Fan Coil Unit, Type A".to_owned()),
            standard_ref: Some("MasterFormat:23 82 16".to_owned()),
            confidence: Some(0.92),
            normalization_source: Some("micro_ml".to_owned()),
            document_id: Some("doc-001".to_owned()),
            page_index: Some(4),
        };
        let json = serde_json::to_string_pretty(&r).unwrap();
        let back: NormalizationRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.record_id, "nr-001");
        assert_eq!(back.raw_value, "FCU-1A");
    }

    #[test]
    fn entity_record_round_trips_via_serde() {
        let e = EntityRecord {
            schema_version: KNOWLEDGE_SCHEMA_VERSION.to_owned(),
            entity_id: "ent-001".to_owned(),
            entity_type: "manufacturer".to_owned(),
            canonical_name: Some("Carrier Corporation".to_owned()),
            aliases: vec!["CARRIER".to_owned(), "Carrier".to_owned()],
            source_bundle_ids: vec!["bundle-001".to_owned()],
            confidence: Some(0.97),
            standard_ref: None,
        };
        let json = serde_json::to_string_pretty(&e).unwrap();
        let back: EntityRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entity_id, "ent-001");
        assert_eq!(back.aliases.len(), 2);
    }

    #[test]
    fn index_record_round_trips_via_serde() {
        let idx = IndexRecord {
            schema_version: KNOWLEDGE_SCHEMA_VERSION.to_owned(),
            index_id: "idx-001".to_owned(),
            bundle_id: "bundle-001".to_owned(),
            page_index: Some(12),
            section_id: Some("23 82 16".to_owned()),
            content_summary: Some("Fan Coil Units schedule".to_owned()),
            entity_refs: vec!["ent-001".to_owned()],
            standard_refs: vec!["MasterFormat:23 82 16".to_owned()],
            indexed_at_utc: None,
        };
        let json = serde_json::to_string_pretty(&idx).unwrap();
        let back: IndexRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.index_id, "idx-001");
        assert_eq!(back.entity_refs.len(), 1);
    }
}
