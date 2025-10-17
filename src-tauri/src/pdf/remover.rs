use lopdf::Document;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[tauri::command]
pub fn delete_pages(path: &str, pages_to_delete: Vec<u32>, output_path: &str) -> Result<(), String> {
    if pages_to_delete.is_empty() {
        return Err("The list of pages to delete cannot be empty.".to_string());
    }

    let input_path = Path::new(path);
    if !input_path.exists() {
        return Err(format!("Input file not found: {}", path));
    }

    if let Some(parent_dir) = Path::new(output_path).parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).map_err(|e| {
                format!(
                    "Failed to create output directory '{}': {}",
                    parent_dir.display(),
                    e
                )
            })?;
        }
    }

    let mut doc = Document::load(path).map_err(|e| format!("Failed to load PDF '{}': {}", path, e))?;

    let page_count = doc.get_pages().len() as u32;
    let mut pages_to_delete_set = BTreeSet::new();
    for page_num in pages_to_delete {
        if page_num == 0 || page_num > page_count {
            return Err(format!(
                "Invalid page number: {}. Page numbers must be between 1 and {}.",
                page_num, page_count
            ));
        }
        pages_to_delete_set.insert(page_num);
    }

    let pages_to_delete_vec: Vec<u32> = pages_to_delete_set.into_iter().collect();
    doc.delete_pages(&pages_to_delete_vec);

    doc.save(output_path)
        .map_err(|e| format!("Failed to save PDF to '{}': {}", output_path, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Document, Object, Stream};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestEnvironment {
        test_dir: PathBuf,
        output_dir: PathBuf,
        input_pdf_path: PathBuf,
    }

    static TEST_RUN_COUNTER: AtomicUsize = AtomicUsize::new(0);

    impl TestEnvironment {
        fn new(test_name: &str) -> Self {
            let unique_suffix = format!("{}", test_name);

            let test_dir = PathBuf::from("target/test_data_remover").join(&unique_suffix);
            let output_dir = PathBuf::from("target/test_output_remover").join(&unique_suffix);

            if test_dir.exists() {
                fs::remove_dir_all(&test_dir).ok();
            }
            if output_dir.exists() {
                fs::remove_dir_all(&output_dir).ok();
            }

            fs::create_dir_all(&test_dir).expect("Failed to create unique test data directory");
            fs::create_dir_all(&output_dir).expect("Failed to create unique test output directory");

            let input_pdf_path = test_dir.join("sample.pdf");
            create_minimal_pdf(input_pdf_path.to_str().unwrap(), 5, "Sample")
                .expect("Setup: Failed to create dummy sample PDF");
            assert!(
                input_pdf_path.exists(),
                "Setup: Dummy PDF does not exist after creation!"
            );

            TestEnvironment {
                test_dir,
                output_dir,
                input_pdf_path,
            }
        }

        fn output_path(&self, filename: &str) -> PathBuf {
            self.output_dir.join(filename)
        }

        fn input_path_str(&self) -> &str {
            self.input_pdf_path.to_str().unwrap()
        }
    }

    impl Drop for TestEnvironment {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.test_dir).ok();
            fs::remove_dir_all(&self.output_dir).ok();
        }
    }

    fn create_minimal_pdf(
        file_path: &str,
        num_pages: u32,
        text_prefix: &str,
    ) -> std::io::Result<()> {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();

        let font_id = doc.add_object(
            dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", },
        );
        let resources_id = doc.add_object(
            dictionary! { "Font" => dictionary! { "F1" => Object::Reference(font_id) }, },
        );

        let mut kids = vec![];
        for i in 1..=num_pages {
            let content_str = format!("BT /F1 12 Tf 100 700 Td ({}-Page {}) Tj ET", text_prefix, i);
            let content_stream = Stream::new(dictionary! {}, content_str.into_bytes());
            let content_id = doc.add_object(content_stream);

            let page_dict = dictionary! {
                "Type" => "Page",
                "Parent" => Object::Reference(pages_id),
                "MediaBox" => Object::Array(vec![0.into(), 0.into(), 612.into(), 792.into()]),
                "Contents" => Object::Reference(content_id),
                "Resources" => Object::Reference(resources_id),
            };
            let page_id = doc.add_object(Object::Dictionary(page_dict));
            kids.push(Object::Reference(page_id));
        }

        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => Object::Array(kids),
                "Count" => Object::Integer(num_pages as i64),
            }),
        );

        let catalog_id = doc.add_object(
            dictionary! { "Type" => "Catalog", "Pages" => Object::Reference(pages_id), },
        );
        doc.trailer.set("Root", Object::Reference(catalog_id));

        doc.save(file_path)?;
        Ok(())
    }

    #[test]
    fn test_delete_pages_success() {
        let env = TestEnvironment::new("delete_success");
        let output_path = env.output_path("deleted_2_4.pdf");
        let pages_to_delete = vec![2, 4];

        let result = delete_pages(
            env.input_path_str(),
            pages_to_delete.clone(),
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok(), "delete_pages failed: {:?}", result.err());
        assert!(
            output_path.exists(),
            "Output file was not created at {}",
            output_path.display()
        );

        match Document::load(&output_path) {
            Ok(output_doc) => {
                assert_eq!(
                    output_doc.get_pages().len(),
                    3,
                    "Output PDF page count mismatch."
                );
                let output_pages = output_doc.get_pages();
                assert!(output_pages.contains_key(&1));
                assert!(output_pages.contains_key(&2));
                assert!(output_pages.contains_key(&3));
                assert!(!output_pages.contains_key(&4));
                assert!(!output_pages.contains_key(&5));
            }
            Err(e) => panic!(
                "Failed to load the generated output PDF '{}': {}",
                output_path.display(),
                e
            ),
        }
    }

    #[test]
    fn test_delete_pages_invalid_page() {
        let env = TestEnvironment::new("delete_invalid_page");
        let output_path = env.output_path("delete_invalid.pdf");
        let pages_to_delete = vec![1, 6];

        let result = delete_pages(
            env.input_path_str(),
            pages_to_delete,
            output_path.to_str().unwrap(),
        );

        assert!(
            result.is_err(),
            "Function should fail for out-of-bounds page"
        );
        if let Err(e) = result {
            assert!(
                e.contains("Invalid page number: 6"),
                "Error message mismatch"
            );
        }
        assert!(!output_path.exists());
    }

    #[test]
    fn test_delete_pages_empty_list() {
        let env = TestEnvironment::new("delete_empty_list");
        let output_path = env.output_path("delete_empty.pdf");
        let pages_to_delete = vec![];

        let result = delete_pages(
            env.input_path_str(),
            pages_to_delete,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_err(), "Function should fail for empty pages list");
        if let Err(e) = result {
            assert!(
                e.contains("The list of pages to delete cannot be empty."),
                "Error message mismatch"
            );
        }
        assert!(!output_path.exists());
    }
}
