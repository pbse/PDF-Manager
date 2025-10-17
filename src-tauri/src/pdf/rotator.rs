use lopdf::{Document, Error as LopdfError, Object, ObjectId};
use std::fs;
use std::path::Path;

#[tauri::command]
pub fn rotate_pdf(path: &str, pages: Vec<u32>, rotation: i32, output_path: &str) -> Result<(), String> {
    if ![0, 90, 180, 270, -90, -180, -270].contains(&rotation) {
        return Err("Invalid rotation angle. Must be one of 0, 90, 180, 270.".to_string());
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

    let page_ids = doc.get_pages();
    let pages_to_rotate: Vec<ObjectId> = if pages.is_empty() {
        page_ids.values().cloned().collect()
    } else {
        pages
            .iter()
            .map(|p| {
                page_ids
                    .get(p)
                    .cloned()
                    .ok_or_else(|| format!("Page {} not found in document.", p))
            })
            .collect::<Result<Vec<ObjectId>, String>>()?
    };

    for page_id in pages_to_rotate {
        let page_dict = doc.get_object_mut(page_id)
            .and_then(|obj| obj.as_dict_mut())
            .map_err(|e: LopdfError| format!("Failed to get page dictionary for page {:?}: {}", page_id, e))?;

        let current_rotation = page_dict
            .get(b"Rotate")
            .and_then(|obj| obj.as_i64())
            .unwrap_or(0) as i32;

        let new_rotation = (current_rotation + rotation) % 360;

        page_dict.set("Rotate", Object::Integer(new_rotation as i64));
    }

    doc.save(output_path)
        .map_err(|e| format!("Failed to save rotated PDF to '{}': {}", output_path, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Document, Object, Stream};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestEnvironment {
        base_name: String,
        run_id: usize,
        test_dir: PathBuf,
        output_dir: PathBuf,
        input_pdf_path: PathBuf,
    }

    static TEST_RUN_COUNTER: AtomicUsize = AtomicUsize::new(0);

    impl TestEnvironment {
        fn new(test_name: &str) -> Self {
            let run_id = TEST_RUN_COUNTER.fetch_add(1, Ordering::SeqCst);
            let unique_suffix = format!("{}_{}", test_name, run_id);

            let test_dir = PathBuf::from("target/test_data_rotator").join(&unique_suffix);
            let output_dir = PathBuf::from("target/test_output_rotator").join(&unique_suffix);

            if test_dir.exists() {
                fs::remove_dir_all(&test_dir).ok();
            }
            if output_dir.exists() {
                fs::remove_dir_all(&output_dir).ok();
            }

            fs::create_dir_all(&test_dir).expect("Failed to create unique test data directory");
            fs::create_dir_all(&output_dir).expect("Failed to create unique test output directory");

            let input_pdf_path = test_dir.join("sample.pdf");
            create_minimal_pdf(input_pdf_path.to_str().unwrap(), 3, "Sample")
                .expect("Setup: Failed to create dummy sample PDF");
            assert!(
                input_pdf_path.exists(),
                "Setup: Dummy PDF does not exist after creation!"
            );

            TestEnvironment {
                base_name: test_name.to_string(),
                run_id,
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
    fn test_rotate_pdf_success() {
        let env = TestEnvironment::new("rotate_success");
        let output_path = env.output_path("rotated_90.pdf");
        let pages_to_rotate = vec![1, 3];

        let result = rotate_pdf(
            env.input_path_str(),
            pages_to_rotate.clone(),
            90,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok(), "rotate_pdf failed: {:?}", result.err());
        assert!(
            output_path.exists(),
            "Output file was not created at {}",
            output_path.display()
        );

        match Document::load(&output_path) {
            Ok(output_doc) => {
                let pages = output_doc.get_pages();
                let page1_id = pages.get(&1).unwrap();
                let page1_dict = output_doc.get_object(*page1_id).unwrap().as_dict().unwrap();
                let rotation1 = page1_dict.get(b"Rotate").unwrap().as_i64().unwrap();
                assert_eq!(rotation1, 90);

                let page2_id = pages.get(&2).unwrap();
                let page2_dict = output_doc.get_object(*page2_id).unwrap().as_dict().unwrap();
                assert!(page2_dict.get(b"Rotate").is_err());

                let page3_id = pages.get(&3).unwrap();
                let page3_dict = output_doc.get_object(*page3_id).unwrap().as_dict().unwrap();
                let rotation3 = page3_dict.get(b"Rotate").unwrap().as_i64().unwrap();
                assert_eq!(rotation3, 90);
            }
            Err(e) => panic!(
                "Failed to load the generated output PDF '{}': {}",
                output_path.display(),
                e
            ),
        }
    }

    #[test]
    fn test_rotate_pdf_invalid_angle() {
        let env = TestEnvironment::new("rotate_invalid_angle");
        let output_path = env.output_path("rotate_invalid.pdf");
        let pages_to_rotate = vec![1];

        let result = rotate_pdf(
            env.input_path_str(),
            pages_to_rotate,
            45,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_err(), "Function should fail for invalid angle");
        if let Err(e) = result {
            assert!(
                e.contains("Invalid rotation angle"),
                "Error message mismatch"
            );
        }
        assert!(!output_path.exists());
    }

    #[test]
    fn test_rotate_pdf_all_pages() {
        let env = TestEnvironment::new("rotate_all_pages");
        let output_path = env.output_path("rotated_all_180.pdf");

        let result = rotate_pdf(
            env.input_path_str(),
            vec![],
            180,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok(), "rotate_pdf failed: {:?}", result.err());
        assert!(
            output_path.exists(),
            "Output file was not created at {}",
            output_path.display()
        );

        match Document::load(&output_path) {
            Ok(output_doc) => {
                let pages = output_doc.get_pages();
                for page_num in 1..=pages.len() {
                    let page_id = pages.get(&(page_num as u32)).unwrap();
                    let page_dict = output_doc.get_object(*page_id).unwrap().as_dict().unwrap();
                    let rotation = page_dict.get(b"Rotate").unwrap().as_i64().unwrap();
                    assert_eq!(rotation, 180);
                }
            }
            Err(e) => panic!(
                "Failed to load the generated output PDF '{}': {}",
                output_path.display(),
                e
            ),
        }
    }
}