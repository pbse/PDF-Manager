use crate::pdf::utils::manual_deep_copy;
use lopdf::{dictionary, Document, Object};
use std::fs;
use std::path::Path;

// --- split_pdf Function using Manual Deep Copy ---
#[tauri::command]
pub fn split_pdf(path: &str, pages: Vec<u32>, output_path: &str) -> Result<(), String> {
    // --- Input Validation & Dir Creation ---
    if pages.is_empty() {
        return Err("The list of pages to extract cannot be empty.".to_string());
    }
    let input_path = Path::new(path);
    if !input_path.exists() {
        return Err(format!("Input file not found: {}", path));
    }
    if !input_path.is_file() {
        return Err(format!("Input path is not a file: {}", path));
    }
    // --- Load Original Document ---
    let doc = Document::load(path).map_err(|e| format!("Failed to load PDF '{}': {}", path, e))?;

    // --- Prepare New Document ---
    let mut new_doc = Document::with_version(doc.version.clone());
    let new_pages_id = new_doc.new_object_id(); // Placeholder for the new Pages node
    let new_catalog_id = new_doc.new_object_id(); // Placeholder for the new Catalog node

    // --- Identify Page Object IDs to Copy ---
    let source_pages_map = doc.get_pages();
    let mut page_ids_to_copy = Vec::with_capacity(pages.len());
    for &page_num in &pages {
        if page_num == 0 {
            return Err(format!(
                "Invalid page number: {}. Page numbers must be 1-based.",
                page_num
            ));
        }
        match source_pages_map.get(&page_num) {
            Some(&page_id) => page_ids_to_copy.push(page_id),
            None => {
                return Err(format!(
                    "Page number {} not found in document '{}' (which has {} pages).",
                    page_num,
                    path,
                    source_pages_map.len()
                ))
            }
        }
    }

    // --- Ensure output directory exists only after inputs are validated ---
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

    // --- Perform Manual Deep Copy for all selected pages ---
    // Map potential LopdfError from deep copy to String
    let object_map = manual_deep_copy(&doc, &mut new_doc, &page_ids_to_copy).map_err(|e| {
        format!(
            "Failed to deep copy pages {:?} from '{}': {}",
            pages, path, e
        )
    })?;

    // --- Build the 'Kids' array and Update Parent Pointers ---
    let mut new_kids = Vec::with_capacity(pages.len());
    for old_page_id in &page_ids_to_copy {
        // Find the corresponding new ObjectId using the map
        let new_page_id = *object_map.get(old_page_id).ok_or_else(|| {
            format!(
                "Internal error: Copied page ObjectId {:?} not found in mapping.",
                old_page_id
            )
        })?;

        new_kids.push(Object::Reference(new_page_id)); // Add ref to new page ID to Kids

        // Update the Parent reference in the copied page object
        // Use a block to limit the mutable borrow
        {
            let page_obj = new_doc.get_object_mut(new_page_id).map_err(|e| {
                format!(
                    "Failed to retrieve copied page object {:?} to update Parent: {}",
                    new_page_id, e
                )
            })?;

            let page_dict = page_obj.as_dict_mut().map_err(|_| {
                format!(
                    "Internal error: Copied page object {:?} is not a dictionary.",
                    new_page_id
                )
            })?;

            page_dict.set("Parent", Object::Reference(new_pages_id)); // Point to the new Pages node ID
        } // Mutable borrow ends here
    }

    // --- Finalize New Document Structure ---
    new_doc.objects.insert(
        new_pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => Object::Array(new_kids), // Use the collected new page references
            "Count" => Object::Integer(pages.len() as i64), // Count is the number of extracted pages
        }),
    );
    new_doc.objects.insert(
        new_catalog_id,
        Object::Dictionary(dictionary! {
            "Type" => "Catalog",
            "Pages" => Object::Reference(new_pages_id), // Reference the new Pages node
        }),
    );
    new_doc
        .trailer
        .set("Root", Object::Reference(new_catalog_id));

    // --- Compress and Save ---
    new_doc.compress();
    new_doc
        .save(output_path)
        .map_err(|e| format!("Failed to save split PDF to '{}': {}", output_path, e))?;

    Ok(())
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*; // Imports split_pdf and its helpers if defined in parent mod
    use crate::pdf::test_utils::create_minimal_pdf;
    use lopdf::Document;
    use std::fs;
    use std::io::Write; // For non-pdf test
    use std::path::{Path, PathBuf};

    // --- RAII Guard for Test Environment ---
    struct TestEnvironment {
        test_dir: PathBuf,
        output_dir: PathBuf,
        // Store the primary input path created by setup
        input_pdf_path: PathBuf,
    }

    impl TestEnvironment {
        fn new(test_name: &str) -> Self {
            let unique_suffix = format!("{}", test_name);

            // Place artifacts in target/ directory
            let test_dir = PathBuf::from("target/test_data_splitter").join(&unique_suffix);
            let output_dir = PathBuf::from("target/test_output_splitter").join(&unique_suffix);

            // Clean up potential remnants from previous runs of THIS specific test
            if test_dir.exists() {
                fs::remove_dir_all(&test_dir).ok();
            }
            if output_dir.exists() {
                fs::remove_dir_all(&output_dir).ok();
            }

            // Create fresh dirs
            fs::create_dir_all(&test_dir).expect("Failed to create unique test data directory");
            fs::create_dir_all(&output_dir).expect("Failed to create unique test output directory");

            // Define and create the primary input PDF
            let input_pdf_path = test_dir.join("sample.pdf");
            // Use the fixed create_minimal_pdf helper
            create_minimal_pdf(input_pdf_path.to_str().unwrap(), 3, "Sample")
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

        // Helper to get the full path for an output file
        fn output_path(&self, filename: &str) -> PathBuf {
            self.output_dir.join(filename)
        }

        // Helper to get the primary input path as str
        fn input_path_str(&self) -> &str {
            self.input_pdf_path.to_str().unwrap()
        }
    }

    // Implement Drop for automatic cleanup
    impl Drop for TestEnvironment {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.test_dir).ok();
            fs::remove_dir_all(&self.output_dir).ok();
        }
    }

    // --- Minimal PDF Creation Helper (Corrected for lopdf 0.31.0) ---
    // --- Updated Tests ---

    #[test]
    fn test_split_pdf_success() {
        let env = TestEnvironment::new("split_success"); // RAII setup/teardown
        let output_path = env.output_path("split_1_3.pdf");
        let pages_to_extract = vec![1, 3];

        // Ensure the output directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let result = split_pdf(
            env.input_path_str(),
            pages_to_extract.clone(),
            output_path.to_str().unwrap(),
        );

        assert!(result.is_ok(), "split_pdf failed: {:?}", result.err());
        assert!(
            output_path.exists(),
            "Output file was not created at {}",
            output_path.display()
        );

        match Document::load(&output_path) {
            Ok(output_doc) => {
                assert_eq!(
                    output_doc.get_pages().len(),
                    pages_to_extract.len(),
                    "Output PDF page count mismatch."
                );
                let output_pages = output_doc.get_pages();
                // Check keys representing the *new* page numbers in the output doc
                assert!(output_pages.contains_key(&1), "Page 1 missing in output");
                assert!(
                    output_pages.contains_key(&2),
                    "Page 2 (original 3) missing in output"
                );
            }
            Err(e) => panic!(
                "Failed to load the generated output PDF '{}': {}",
                output_path.display(),
                e
            ),
        }
    }

    #[test]
    fn test_split_pdf_invalid_page() {
        let env = TestEnvironment::new("split_invalid_page");
        let output_path = env.output_path("split_invalid.pdf");
        let pages_to_extract = vec![1, 5]; // Page 5 invalid for a 3-page doc

        let result = split_pdf(
            env.input_path_str(),
            pages_to_extract,
            output_path.to_str().unwrap(),
        );

        assert!(
            result.is_err(),
            "Function should fail for out-of-bounds page"
        );
        if let Err(e) = result {
            println!("Expected error: {}", e);
            assert!(
                e.contains("Page number 5 not found"),
                "Error message mismatch"
            );
        }
        assert!(
            !output_path.exists() || Document::load(&output_path).is_err(),
            "Output file should ideally not exist or be invalid after an error."
        );
    }

    #[test]
    fn test_split_pdf_empty_pages() {
        let env = TestEnvironment::new("split_empty_pages");
        let output_path = env.output_path("split_empty.pdf");
        let pages_to_extract = vec![];

        let result = split_pdf(
            env.input_path_str(),
            pages_to_extract,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_err(), "Function should fail for empty pages list");
        if let Err(e) = result {
            println!("Expected error: {}", e);
            assert!(e.contains("cannot be empty"), "Error message mismatch");
        }
        assert!(
            !output_path.exists() || Document::load(&output_path).is_err(),
            "Output file should ideally not exist or be invalid after an error."
        );
    }

    #[test]
    fn test_split_pdf_input_not_found() {
        // Don't need full env, just a guaranteed non-existent path
        let bad_input_path = "target/test_data_splitter/non_existent_dir/no_way_this_exists.pdf";
        fs::remove_file(bad_input_path).ok();
        if let Some(parent) = Path::new(bad_input_path).parent() {
            fs::remove_dir_all(parent).ok();
        }

        let output_path = "target/test_output_splitter/output_for_bad_input_split.pdf";
        // Ensure output dir exists so function doesn't fail on *that*
        if let Some(parent) = Path::new(output_path).parent() {
            fs::create_dir_all(parent).ok();
        }

        let pages_to_extract = vec![1];
        let result = split_pdf(bad_input_path, pages_to_extract, output_path);

        assert!(
            result.is_err(),
            "Function should fail if input file not found"
        );
        if let Err(e) = result {
            println!("Expected error: {}", e);
            // Check for the error coming from the initial exists() check OR Document::load
            assert!(
                e.contains("Input file not found")
                    || (e.contains("Failed to load PDF") && e.contains("No such file")),
                "Error message mismatch"
            );
        }

        // Clean up output dir if created
        if let Some(parent) = Path::new(output_path).parent() {
            fs::remove_dir_all(parent).ok();
        }
    }

    #[test]
    fn test_split_pdf_zero_page() {
        let env = TestEnvironment::new("split_page_zero");
        let output_path = env.output_path("split_zero.pdf");
        let pages_to_extract = vec![0, 1]; // Page 0 is invalid

        let result = split_pdf(
            env.input_path_str(),
            pages_to_extract,
            output_path.to_str().unwrap(),
        );

        assert!(result.is_err(), "Function should fail for page zero");
        if let Err(e) = result {
            println!("Expected error: {}", e);
            assert!(
                e.contains("Page numbers must be 1-based") || e.contains("Invalid page number: 0"),
                "Error message mismatch"
            );
        }
        assert!(
            !output_path.exists() || Document::load(&output_path).is_err(),
            "Output file should ideally not exist or be invalid after an error."
        );
        // Clean up output dir if created
        if let Some(parent) = Path::new(&output_path).parent() {
            fs::remove_dir_all(parent).ok();
        }
    }

    #[test]
    fn test_split_input_not_a_pdf() {
        let env = TestEnvironment::new("split_not_a_pdf");
        let not_pdf_path = env.test_dir.join("not_a_pdf.txt");
        let output_path = env.output_path("output_for_not_pdf_split.pdf");

        let mut file = fs::File::create(&not_pdf_path).expect("Failed to create dummy text file");
        writeln!(file, "This is text.").expect("Failed to write to text file");
        assert!(not_pdf_path.exists());

        let pages_to_extract = vec![1];
        let result = split_pdf(
            not_pdf_path.to_str().unwrap(),
            pages_to_extract,
            output_path.to_str().unwrap(),
        );

        assert!(
            result.is_err(),
            "Function should fail if input is not a PDF"
        );
        if let Err(e) = result {
            println!("Expected error: {}", e);
            assert!(
                e.contains("Failed to load PDF")
                    || e.contains("invalid PDF header")
                    || e.contains("cannot find trailer"),
                "Error message mismatch"
            );
        }
        assert!(
            !output_path.exists() || Document::load(&output_path).is_err(),
            "Output file should ideally not exist or be invalid after an error."
        );
        // Clean up output dir if created
        if let Some(parent) = Path::new(&output_path).parent() {
            fs::remove_dir_all(parent).ok();
        }
    }
}
