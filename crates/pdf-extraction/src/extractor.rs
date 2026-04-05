//! PDF extractor trait and implementations.
//!
//! This module defines the abstraction for PDF extraction operations, allowing
//! different PDF libraries to implement a common interface.

use crate::error::Result;
use crate::types::{Document, PageData};
use log::debug;
use pdfium_render::prelude::*;
use std::env;
use std::path::Path;

/// Trait for PDF extraction implementations.
///
/// This trait abstracts over different PDF processing libraries, providing a common
/// interface for document loading, metadata retrieval, and page extraction.
///
/// Implementations should handle format-specific details transparently.
pub trait PdfExtractor {
    /// Loads a PDF document from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PDF file
    ///
    /// # Returns
    ///
    /// A handle to the loaded document, or an error if loading fails
    ///
    /// # Errors
    ///
    /// Returns an error when the PDF cannot be opened or parsed.
    fn load_document(&self, path: &str) -> Result<Document>;

    /// Gets the total number of pages in a document.
    ///
    /// # Arguments
    ///
    /// * `doc` - Reference to the document
    ///
    /// # Returns
    ///
    /// The total page count
    fn get_page_count(&self, doc: &Document) -> usize;

    /// Extracts content from a specific page.
    ///
    /// # Arguments
    ///
    /// * `doc` - Reference to the document
    /// * `page_index` - Zero-based page index
    ///
    /// # Returns
    ///
    /// Extracted page data, or an error if extraction fails
    ///
    /// # Errors
    ///
    /// Returns an error when the page cannot be read or decoded.
    fn extract_page(&self, doc: &Document, page_index: usize) -> Result<PageData>;

    /// Extracts text from a specific page.
    ///
    /// # Arguments
    ///
    /// * `doc` - Reference to the document
    /// * `page_index` - Zero-based page index
    ///
    /// # Returns
    ///
    /// The extracted text from the page, or an error if extraction fails
    ///
    /// # Errors
    ///
    /// Returns an error when the page index is out of bounds or text extraction fails.
    fn extract_text(&self, doc: &Document, page_index: usize) -> Result<String>;
}

/// PDF extractor implementation using Pdfium.
///
/// This struct provides PDF extraction capabilities using the pdfium-render library.
pub struct PdfiumExtractor {
    pdfium: Pdfium,
}

impl PdfiumExtractor {
    /// Creates a new `PdfiumExtractor` instance.
    ///
    /// # Returns
    ///
    /// A new extractor with Pdfium library initialized.
    ///
    /// # Panics
    ///
    /// Panics if `PDFium` library cannot be found or initialized.
    #[must_use]
    pub fn new() -> Self {
        let pdfium =
            Self::load_pdfium().unwrap_or_else(|e| panic!("Failed to initialize PDFium: {e}"));
        Self { pdfium }
    }

    /// Creates a new `PdfiumExtractor`, returning an error instead of panicking if PDFium
    /// cannot be found or initialized.
    ///
    /// # Errors
    ///
    /// Returns a descriptive error string when PDFium cannot be located or bound.
    pub fn try_new() -> std::result::Result<Self, String> {
        let pdfium = Self::load_pdfium()?;
        Ok(Self { pdfium })
    }

    /// Attempts to load `PDFium` from various locations.
    fn load_pdfium() -> std::result::Result<Pdfium, String> {
        // Try PDFIUM_LIB_PATH environment variable
        if let Ok(dir) = env::var("PDFIUM_LIB_PATH") {
            if let Ok(bindings) =
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&dir))
            {
                return Ok(Pdfium::new(bindings));
            }
        }

        // Try workspace root (for tests)
        if let Ok(workspace_root) = env::var("CARGO_WORKSPACE_DIR") {
            if let Ok(bindings) = Pdfium::bind_to_library(
                Pdfium::pdfium_platform_library_name_at_path(&workspace_root),
            ) {
                return Ok(Pdfium::new(bindings));
            }
        }

        // Try current working directory
        if let Ok(cwd) = env::current_dir() {
            if let Ok(bindings) = Pdfium::bind_to_library(
                Pdfium::pdfium_platform_library_name_at_path(cwd.to_str().unwrap_or(".")),
            ) {
                return Ok(Pdfium::new(bindings));
            }
        }

        // Try project root directory (crates/pdf-extraction goes up 2 levels)
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let mut root = std::path::PathBuf::from(manifest_dir);
        root.pop(); // crates/pdf-extraction
        root.pop(); // crates
        if let Ok(bindings) = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
            root.to_str().unwrap_or("."),
        )) {
            return Ok(Pdfium::new(bindings));
        }

        // Try system library
        if let Ok(bindings) = Pdfium::bind_to_system_library() {
            return Ok(Pdfium::new(bindings));
        }

        Err("PDFium library not found. Download pdfium.dll/dylib/so and place it in project root or set PDFIUM_LIB_PATH".to_string())
    }
}

impl Default for PdfiumExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfExtractor for PdfiumExtractor {
    fn load_document(&self, path: &str) -> Result<Document> {
        debug!("Loading PDF from {path}");

        // Check if path is empty
        if path.is_empty() {
            return Err(crate::error::ExtractionError::invalid_path("empty path"));
        }

        // Check if file exists
        let file_path = Path::new(path);
        if !file_path.exists() {
            return Err(crate::error::ExtractionError::file_not_found(path));
        }

        // Check file extension
        if !path.to_lowercase().ends_with(".pdf") {
            return Err(crate::error::ExtractionError::invalid_format(
                "file must have .pdf extension",
            ));
        }

        // Load the document
        let document = self
            .pdfium
            .load_pdf_from_file(path, None)
            .map_err(|e| crate::error::ExtractionError::pdf_error(format!("{e}")))?;

        let page_count = document.pages().len() as usize;

        Ok(Document::new(path.to_string(), page_count))
    }

    fn get_page_count(&self, doc: &Document) -> usize {
        let count = doc.page_count;
        debug!("PDF has {count} pages");
        count
    }

    fn extract_page(&self, doc: &Document, page_index: usize) -> Result<PageData> {
        if page_index >= doc.page_count {
            return Err(crate::error::ExtractionError::page_not_found(page_index));
        }

        let document = self.pdfium.load_pdf_from_file(&doc.path, None).map_err(|e| {
            crate::error::ExtractionError::pdf_error(format!(
                "Failed to load document for page extraction: {e}"
            ))
        })?;

        #[allow(clippy::cast_possible_truncation)]
        let page_index_u16: u16 = page_index as u16;

        let page = document.pages().get(page_index_u16).map_err(|e| {
            crate::error::ExtractionError::pdf_error(format!(
                "Failed to get page {page_index}: {e}"
            ))
        })?;

        let width_pts = page.width().value;
        let height_pts = page.height().value;

        let mut spans: Vec<crate::types::SpanData> = Vec::new();

        for object in page.objects().iter() {
            if let Some(text_obj) = object.as_text_object() {
                let text = text_obj.text();

                // Skip empty/whitespace-only text objects — they carry no content.
                if text.trim().is_empty() {
                    continue;
                }

                let font_size = text_obj.scaled_font_size().value;
                let font_name = text_obj.font().name();

                // bounds() returns PdfQuadPoints in PDF coordinates (bottom-left origin).
                let bbox = match object.bounds() {
                    Ok(qp) => crate::types::RawBBox {
                        x: qp.left().value,
                        y: qp.bottom().value,
                        width: (qp.right().value - qp.left().value).abs(),
                        height: (qp.top().value - qp.bottom().value).abs(),
                    },
                    Err(_) => {
                        // Log and skip objects whose bounds cannot be determined.
                        debug!(
                            "Skipping text object on page {page_index}: bounds unavailable"
                        );
                        continue;
                    }
                };

                spans.push(crate::types::SpanData { text, bbox, font_size, font_name });
            }
        }

        let span_count = spans.len();
        debug!("Extracted {span_count} spans from page {page_index} ({width_pts}x{height_pts}pt)");

        Ok(crate::types::PageData { page_index, width_pts, height_pts, spans })
    }

    fn extract_text(&self, doc: &Document, page_index: usize) -> Result<String> {
        // Check if page index is valid
        if page_index >= doc.page_count {
            return Err(crate::error::ExtractionError::page_not_found(page_index));
        }

        // Load the document to extract text
        let document = self.pdfium.load_pdf_from_file(&doc.path, None).map_err(|e| {
            crate::error::ExtractionError::pdf_error(format!(
                "Failed to load document for text extraction: {e}"
            ))
        })?;

        // Convert page_index to u16 for pdfium-render API
        #[allow(clippy::cast_possible_truncation)]
        let page_index_u16: u16 = page_index as u16;

        // Get the page
        let page = document.pages().get(page_index_u16).map_err(|e| {
            crate::error::ExtractionError::pdf_error(format!(
                "Failed to get page {page_index}: {e}"
            ))
        })?;

        // Extract text from the page
        let text = page
            .text()
            .map_err(|e| {
                crate::error::ExtractionError::pdf_error(format!("Failed to extract text: {e}"))
            })?
            .all();

        let text_len = text.len();
        debug!("Extracted {text_len} chars from page {page_index}");

        Ok(text)
    }
}

impl PdfiumExtractor {
    /// Renders a single PDF page as a PNG with coloured rectangles drawn over each
    /// span bounding box from the IR.
    ///
    /// The image is rendered at 1400 px wide (aspect-ratio preserved).  Each span
    /// receives a 2-px green outline.  Coordinates in `span_bboxes` use the
    /// normalised top-left origin convention (`[0.0, 1.0]`).
    ///
    /// # Errors
    ///
    /// Returns an [`ExtractionError`] if the PDF cannot be loaded, the page cannot
    /// be rendered, or saving the output image fails.
    pub fn render_page_with_spans(
        &self,
        pdf_path: &str,
        page_index: usize,
        span_bboxes: &[conset_pdf_ir::BBox],
        output_path: &std::path::Path,
    ) -> Result<()> {
        use pdfium_render::prelude::PdfRenderConfig;

        let document =
            self.pdfium.load_pdf_from_file(pdf_path, None).map_err(|e| {
                crate::error::ExtractionError::pdf_error(format!(
                    "Failed to load PDF for rendering: {e}"
                ))
            })?;

        if page_index >= document.pages().len() as usize {
            return Err(crate::error::ExtractionError::page_not_found(page_index));
        }

        #[allow(clippy::cast_possible_truncation)]
        let page_index_u16 = page_index as u16;

        let page = document.pages().get(page_index_u16).map_err(|e| {
            crate::error::ExtractionError::pdf_error(format!("Failed to get page: {e}"))
        })?;

        let render_config = PdfRenderConfig::new()
            .set_target_width(1400)
            .render_form_data(true)
            .render_annotations(false);

        let bitmap = page.render_with_config(&render_config).map_err(|e| {
            crate::error::ExtractionError::pdf_error(format!("Page render failed: {e}"))
        })?;

        let mut img = bitmap.as_image().into_rgba8();
        let green = image::Rgba([0u8, 200u8, 0u8, 230u8]);
        let fw = f64::from(img.width());
        let fh = f64::from(img.height());

        for bbox in span_bboxes {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let (bx, by, bw, bh) = (
                (bbox.x * fw) as u32,
                (bbox.y * fh) as u32,
                ((bbox.width * fw) as u32).max(1),
                ((bbox.height * fh) as u32).max(1),
            );
            draw_rect_outline(&mut img, bx, by, bw, bh, green);
        }

        img.save(output_path).map_err(|e| {
            crate::error::ExtractionError::pdf_error(format!("Failed to save overlay PNG: {e}"))
        })?;

        debug!("Saved overlay for page {page_index} → {}", output_path.display());
        Ok(())
    }
}

/// Draws a 2-px green outline rectangle on `img`.
///
/// All coordinates are clamped to image bounds so that no out-of-bounds pixel
/// access can occur.
fn draw_rect_outline(
    img: &mut image::RgbaImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: image::Rgba<u8>,
) {
    let img_w = img.width();
    let img_h = img.height();

    let put = |img: &mut image::RgbaImage, px: u32, py: u32| {
        if px < img_w && py < img_h {
            img.put_pixel(px, py, color);
        }
    };

    // Top and bottom edges
    let x_end = x.saturating_add(w);
    for px in x..=x_end {
        put(img, px, y);
        put(img, px, y.saturating_add(h));
        // 2-px thickness
        put(img, px, y.saturating_add(1));
        put(img, px, y.saturating_add(h).saturating_add(1));
    }
    // Left and right edges (skip corners already drawn)
    let y_end = y.saturating_add(h);
    for py in y..=y_end {
        put(img, x, py);
        put(img, x_end, py);
        put(img, x.saturating_add(1), py);
        put(img, x_end.saturating_add(1), py);
    }
}
