use lopdf::{Document, Object, content::Content};
use docx_rs::*;
use std::fs::File;

#[tauri::command]
pub fn replace_text_block(
    path: &str,
    page_num: u32,
    old_text: &str,
    new_text: &str,
    output_path: &str,
) -> Result<(), String> {
    let mut doc = Document::load(path).map_err(|e| e.to_string())?;
    let pages = doc.get_pages();
    let page_id = *pages.get(&page_num).ok_or("Page not found")?;

    let content_data = doc.get_page_content(page_id).map_err(|e| e.to_string())?;
    let mut content = Content::decode(&content_data).map_err(|e| e.to_string())?;

    let mut found = false;
    for operation in &mut content.operations {
        if operation.operator == "Tj" || operation.operator == "TJ" {
            for arg in &mut operation.operands {
                match arg {
                    Object::String(bytes, _) => {
                        let text = String::from_utf8_lossy(bytes);
                        if text.contains(old_text) {
                            let replaced = text.replace(old_text, new_text);
                            *arg = Object::string_literal(replaced);
                            found = true;
                        }
                    }
                    Object::Array(items) => {
                        for item in items {
                            if let Object::String(bytes, _) = item {
                                let text = String::from_utf8_lossy(bytes);
                                if text.contains(old_text) {
                                    let replaced = text.replace(old_text, new_text);
                                    *item = Object::string_literal(replaced);
                                    found = true;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if !found {
        return Err("Text block not found in the specified page. It might be split into multiple streams.".to_string());
    }

    let encoded_content = content.encode().map_err(|e| e.to_string())?;
    doc.change_page_content(page_id, encoded_content).map_err(|e| e.to_string())?;

    doc.save(output_path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn pdf_to_docx(path: &str, output_path: &str) -> Result<(), String> {
    // 1. Extract text
    let text = pdf_extract::extract_text(path).map_err(|e| format!("Extraction failed: {:?}", e))?;
    
    let file = File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut docx = Docx::new();
    
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_text(line)));
    }

    docx.build().pack(file).map_err(|e| format!("Docx build failed: {:?}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::test_utils::{create_minimal_pdf, setup_unique_paths, teardown_unique_paths};

    #[test]
    fn test_pdf_to_docx_success() {
        let (test_dir, output_dir) = setup_unique_paths("docx");
        let input = test_dir.join("input.pdf");
        let output = output_dir.join("output.docx");

        create_minimal_pdf(input.to_str().unwrap(), 1, "DocxTest").unwrap();

        let result = pdf_to_docx(input.to_str().unwrap(), output.to_str().unwrap());
        assert!(result.is_ok());
        assert!(output.exists());

        teardown_unique_paths(&test_dir, &output_dir);
    }
}
