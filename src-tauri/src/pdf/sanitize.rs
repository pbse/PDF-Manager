use lopdf::{Document, Object};
use std::path::Path;
use std::fs;

#[tauri::command]
pub fn sanitize_pdf(path: &str, output_path: &str) -> Result<(), String> {
    let input_path = Path::new(path);
    if !input_path.exists() || !input_path.is_file() {
        return Err(format!("Input file not found: {}", path));
    }
    if let Some(parent_dir) = Path::new(output_path).parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).map_err(|e| {
                format!("Failed to create output directory: {}", e)
            })?;
        }
    }

    let mut doc = Document::load(path).map_err(|e| format!("Failed to load PDF: {}", e))?;

    // 1. Remove Info dictionary from Trailer
    doc.trailer.remove(b"Info");

    // 2. Remove Metadata from Catalog
    if let Ok(Object::Reference(root_id)) = doc.trailer.get(b"Root") {
        if let Ok(Object::Dictionary(catalog)) = doc.get_object(*root_id) {
            let mut new_catalog = catalog.clone();
            new_catalog.remove(b"Metadata");
            // Set the object, ignore if it fails
            let _ = doc.set_object(*root_id, Object::Dictionary(new_catalog));
        }
    }

    doc.save(output_path).map_err(|e| format!("Failed to save sanitized PDF: {}", e))?;
    Ok(())
}
