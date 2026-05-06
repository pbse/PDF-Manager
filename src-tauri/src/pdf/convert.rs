use std::path::Path;
use std::fs;
use pdf_extract::extract_text;

#[tauri::command]
pub fn pdf_to_text(path: &str, output_path: &str) -> Result<(), String> {
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

    let out_text = extract_text(path).map_err(|e| format!("Failed to extract text: {:?}", e))?;
    fs::write(output_path, out_text).map_err(|e| format!("Failed to write to file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn pdf_to_text_string(path: &str) -> Result<String, String> {
    let input_path = Path::new(path);
    if !input_path.exists() || !input_path.is_file() {
        return Err(format!("Input file not found: {}", path));
    }

    let out_text = extract_text(path).map_err(|e| format!("Failed to extract text: {:?}", e))?;
    Ok(out_text)
}

#[tauri::command]
pub fn write_text_file(path: &str, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("Failed to write text to file: {}", e))
}
