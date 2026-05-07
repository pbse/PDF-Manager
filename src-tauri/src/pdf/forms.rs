use lopdf::{Document, Object};

#[derive(serde::Serialize)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: String,
}

#[tauri::command]
pub fn get_form_fields(path: &str) -> Result<Vec<FormField>, String> {
    let doc = Document::load(path).map_err(|e| e.to_string())?;
    let mut fields = vec![];

    if let Ok(Object::Reference(root_id)) = doc.trailer.get(b"Root") {
        if let Ok(Object::Dictionary(catalog)) = doc.get_object(*root_id) {
            if let Ok(Object::Reference(acroform_id)) = catalog.get(b"AcroForm") {
                if let Ok(Object::Dictionary(acroform)) = doc.get_object(*acroform_id) {
                    if let Ok(Object::Array(field_refs)) = acroform.get(b"Fields") {
                        for field_ref in field_refs {
                            if let Ok(fid) = field_ref.as_reference() {
                                if let Ok(Object::Dictionary(field)) = doc.get_object(fid) {
                                    let name = if let Ok(Object::String(bytes, _)) = field.get(b"T") {
                                        String::from_utf8_lossy(bytes).to_string()
                                    } else {
                                        "Unnamed".to_string()
                                    };

                                    let field_type = if let Ok(Object::Name(ft)) = field.get(b"FT") {
                                        String::from_utf8_lossy(ft).to_string()
                                    } else {
                                        "Unknown".to_string()
                                    };

                                    let value = if let Ok(Object::String(bytes, _)) = field.get(b"V") {
                                        String::from_utf8_lossy(bytes).to_string()
                                    } else {
                                        "".to_string()
                                    };

                                    fields.push(FormField { name, field_type, value });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(fields)
}

#[tauri::command]
pub fn set_form_fields(path: &str, updates: std::collections::HashMap<String, String>, output_path: &str) -> Result<(), String> {
    let mut doc = Document::load(path).map_err(|e| e.to_string())?;
    let mut field_updates = vec![];

    if let Ok(Object::Reference(root_id)) = doc.trailer.get(b"Root") {
        if let Ok(Object::Dictionary(catalog)) = doc.get_object(*root_id) {
            if let Ok(Object::Reference(acroform_id)) = catalog.get(b"AcroForm") {
                if let Ok(Object::Dictionary(acroform)) = doc.get_object(*acroform_id) {
                    if let Ok(Object::Array(field_refs)) = acroform.get(b"Fields") {
                        for field_ref in field_refs {
                            if let Ok(fid) = field_ref.as_reference() {
                                if let Ok(Object::Dictionary(field)) = doc.get_object(fid) {
                                    let name = if let Ok(Object::String(bytes, _)) = field.get(b"T") {
                                        String::from_utf8_lossy(bytes).to_string()
                                    } else {
                                        continue;
                                    };

                                    if let Some(new_value) = updates.get(&name) {
                                        let mut new_field = field.clone();
                                        new_field.set("V", Object::string_literal(new_value.as_str()));
                                        new_field.remove(b"AP");
                                        field_updates.push((fid, new_field));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for (fid, field) in field_updates {
        doc.objects.insert(fid, Object::Dictionary(field));
    }

    doc.save(output_path).map_err(|e| format!("Failed to save: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::test_utils::{create_minimal_pdf, setup_unique_paths, teardown_unique_paths};

    #[test]
    fn test_get_form_fields_empty() {
        let (test_dir, output_dir) = setup_unique_paths("forms");
        let input_path = test_dir.join("input.pdf");

        create_minimal_pdf(input_path.to_str().unwrap(), 1, "FormsTest").unwrap();

        let result = get_form_fields(input_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        teardown_unique_paths(&test_dir, &output_dir);
    }
}
