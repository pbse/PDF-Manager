pub mod utils;

// Declare the submodules within the 'pdf' module
pub mod extractor;
pub mod merger;
pub mod parser;
pub mod splitter;

pub mod rotator;

pub mod remover;
pub mod sanitize;
pub mod convert;
pub mod reorder;
pub mod security_utils;
pub mod compare;
pub mod metadata;
pub mod outline;
pub mod annotation_reader;
pub mod forms;
pub mod image_to_pdf;
pub mod watermark;
pub mod watcher;
pub mod forensic_redact;
pub mod templates;
pub mod briefing;

// Shared helpers only compiled for tests
#[cfg(test)]
pub mod test_utils;

pub mod annotations;
pub mod signatures;

// Optional but recommended: Re-export the functions you want to be easily accessible
// from the 'pdf' module itself, hiding the internal structure (parser, merger, etc.)
// This makes the import in main.rs cleaner.
pub use extractor::extract_pdf_page;
pub use merger::merge_pdfs;
pub use parser::parse_pdf;
pub use splitter::split_pdf;
pub use rotator::rotate_pdf;
pub use remover::delete_pages;
pub use sanitize::sanitize_pdf;
pub use convert::pdf_to_text;
pub use convert::pdf_to_text_string;
pub use convert::write_text_file;
pub use annotations::{add_annotation, delete_annotation, update_annotation_contents};
pub use reorder::reorder_pages;
pub use security_utils::{compress_pdf, decrypt_pdf, flatten_annotations};
pub use compare::compare_pdfs_text;
pub use outline::{get_pdf_outline, set_pdf_outline};
pub use annotation_reader::get_annotations;
pub use forms::{get_form_fields, set_form_fields};
pub use image_to_pdf::images_to_pdf;
pub use watermark::add_watermark;
pub use watcher::start_folder_watcher;
pub use forensic_redact::forensic_redact;
pub use templates::markdown_to_pdf;
pub use briefing::generate_briefing;
pub use metadata::{update_metadata, batch_update_metadata};
pub use signatures::add_signature_visual;
pub use signatures::sign_pdf_pfx;
pub use signatures::verify_signatures;
