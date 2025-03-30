// Declare the submodules within the 'pdf' module
pub mod extractor;
pub mod merger;
pub mod parser;
pub mod splitter;

// Optional but recommended: Re-export the functions you want to be easily accessible
// from the 'pdf' module itself, hiding the internal structure (parser, merger, etc.)
// This makes the import in main.rs cleaner.
pub use extractor::extract_pdf_page;
pub use merger::merge_pdfs;
pub use parser::parse_pdf;
pub use splitter::split_pdf;