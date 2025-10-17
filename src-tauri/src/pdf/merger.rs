// Necessary imports
use lopdf::{Document, Error as LopdfError, Object, ObjectId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

// --- Deep Copy Helper Functions (Adapted for Merging/ID Shifting - WITH FIXES) ---

/// Copies objects and dependencies from source_doc to target_doc, shifting ObjectIds.
/// Updates internal references and page parent pointers.
fn merge_and_shift_objects(
    target_doc: &mut Document,
    source_doc: &Document,
    id_offset: u32,
    target_pages_id: ObjectId,
) -> Result<HashMap<ObjectId, ObjectId>, LopdfError> { // Returns LopdfError
    let mut id_map: HashMap<ObjectId, ObjectId> = HashMap::new();
    let mut queue: VecDeque<ObjectId> = VecDeque::new();
    let mut processed: HashSet<ObjectId> = HashSet::new();
    let mut loop_count = 0;
    // Estimate max loops needed more accurately - based only on source objects
    let max_loops = source_doc.objects.len() * 2 + 10; // Added buffer

    // Start queue with all *root* pages from the source document's page tree.
    // This is more efficient than queuing *all* objects initially.
    for page_id in source_doc.get_pages().values() {
        if processed.insert(*page_id) {
            queue.push_back(*page_id);
        }
    }
    // Also add other essential roots like Fonts, XObjects directly referenced by pages?
    // The current recursive find should catch them from the page->resources path.

    // --- Pass 1: Copy objects and build ID map ---
    while let Some(old_id) = queue.pop_front() {
        loop_count += 1;
        if loop_count > max_loops {
            return Err(LopdfError::Syntax(format!("Deep copy loop exceeded limit ({}) for merge", max_loops)));
        }
        if id_map.contains_key(&old_id) { continue; } // Already copied and mapped

        let source_object = match source_doc.get_object(old_id) {
             Ok(obj) => obj,
             Err(e) => {
                 eprintln!("Warning: Failed to get source object {:?} during merge copy: {}. Skipping.", old_id, e);
                 continue;
             }
         };

        // Find references in the ORIGINAL source object FIRST
        find_references_recursive_merge(source_object, &mut queue, &mut processed)?;

        let cloned_object = source_object.clone();
        let new_id = (old_id.0 + id_offset, old_id.1);
        target_doc.objects.insert(new_id, cloned_object);
        id_map.insert(old_id, new_id);
    }

    // --- Pass 2: Update references ---
    for (_old_id, new_id) in &id_map {
        match target_doc.objects.get_mut(new_id) {
            Some(target_object) => {
                update_references_recursive_merge(target_object, &id_map, target_pages_id)?;
            }
            None => { // Changed from get_object_mut(...).is_err() check
                // This indicates an object that *was* mapped couldn't be retrieved. Critical.
                eprintln!("ERROR: Could not get mapped object {:?} for ref update (Pass 2) during merge.", new_id);
                return Err(LopdfError::Syntax(format!("Failed to retrieve copied object {:?} during merge reference update", new_id)));
            }
        }
    }
    Ok(id_map)
}

/// Helper to find Object::Reference IDs within an object (Merge version).
fn find_references_recursive_merge(
    object: &Object,
    queue: &mut VecDeque<ObjectId>,
    processed: &mut HashSet<ObjectId>,
) -> Result<(), LopdfError> {
    // This helper doesn't directly need Dictionary/Stream types, only Object variants
    match object {
        Object::Reference(id) => { if processed.insert(*id) { queue.push_back(*id); } }
        Object::Array(arr) => { for item in arr { find_references_recursive_merge(item, queue, processed)?; } }
        Object::Dictionary(dict) => { for (_, value) in dict.iter() { find_references_recursive_merge(value, queue, processed)?; } }
        Object::Stream(stream) => { for (_, value) in stream.dict.iter() { find_references_recursive_merge(value, queue, processed)?; } }
        _ => {}
    }
    Ok(())
}

/// Helper to update Object::Reference IDs within a mutable object (Merge version).
/// Also updates /Parent in Page dictionaries.
fn update_references_recursive_merge(
    object: &mut Object,
    id_map: &HashMap<ObjectId, ObjectId>,
    target_pages_id: ObjectId,
) -> Result<(), LopdfError> {
     match object {
        Object::Reference(ref mut old_id_ref) => {
            if let Some(new_id) = id_map.get(old_id_ref) { *old_id_ref = *new_id; }
        }
        Object::Array(arr) => {
            // *** FIXED: Use iter_mut() ***
            for item in arr.iter_mut() {
                update_references_recursive_merge(item, id_map, target_pages_id)?;
            }
        }
        Object::Dictionary(dict) => {
            let is_page = match dict.get(b"Type") {
                Ok(Object::Name(ref name)) if name == b"Page" => true,
                _ => false,
            };
            if is_page { dict.set("Parent", Object::Reference(target_pages_id)); }

            // Use iter_mut()
            for (key, value) in dict.iter_mut() {
                if !(is_page && key == b"Parent".as_ref()) {
                    update_references_recursive_merge(value, id_map, target_pages_id)?;
                }
            }
        }
        Object::Stream(stream) => {
             let is_page_stream_dict = match stream.dict.get(b"Type") {
                 Ok(Object::Name(ref name)) if name == b"Page" => true,
                 _ => false,
             };
             if is_page_stream_dict { stream.dict.set("Parent", Object::Reference(target_pages_id)); }

             // Use iter_mut()
             for (key, value) in stream.dict.iter_mut() {
                 if !(is_page_stream_dict && key == b"Parent".as_ref()) {
                    update_references_recursive_merge(value, id_map, target_pages_id)?;
                 }
            }
        }
        _ => {}
    }
    Ok(())
}


// --- merge_pdfs Function ---

#[tauri::command]
pub fn merge_pdfs(paths: Vec<&str>, output_path: &str) -> Result<(), String> {
    // --- Input Validation & Dir Creation (as before) ---
    if paths.is_empty() { return Err("No PDF files provided for merging.".to_string()); }
    if paths.len() == 1 {
        let source_path = paths[0];
        let p = Path::new(source_path);
        if !p.exists() {
            return Err(format!("Input file not found: {}", source_path));
        }
        if !p.is_file() {
            return Err(format!("Input path is not a file: {}", source_path));
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
        fs::copy(source_path, output_path)
            .map(|_| ())
            .map_err(|e| {
                format!(
                    "Failed to copy single PDF from '{}' to '{}': {}",
                    source_path, output_path, e
                )
            })?;
        return Ok(());
    }
    for path in &paths {
        let p = Path::new(path);
        if !p.exists() {
            return Err(format!("Input file not found: {}", path));
        }
        if !p.is_file() {
            return Err(format!("Input path is not a file: {}", path));
        }
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

    // --- Load First Document ---
    let first_path = paths[0];
    let mut target_doc = Document::load(first_path)
         .map_err(|e| format!("Failed to load base PDF '{}': {}", first_path, e))?;

    // --- Get Target Document's Pages Info ---
    let catalog = target_doc.catalog() // Use immutable catalog first
         .map_err(|e| format!("Failed to get catalog from '{}': {}", first_path, e))?;
    // Get the ObjectId of the root /Pages node in the target document
    let target_pages_id = catalog.get(b"Pages")
         .map_err(|e| format!("Failed to get Pages entry from catalog in '{}': {}", first_path, e))?
         .as_reference()
         .map_err(|e| format!("Root Pages entry is not a reference in '{}': {}", first_path, e))?;

    

    // --- Iterate and Merge Remaining Documents ---
    let mut new_kids_data = Vec::new(); // Collect new page references here
    let mut current_max_id = target_doc.max_id; // Track starting from the first doc's max_id

    for path in paths.iter().skip(1) {
         let source_doc = Document::load(*path)
              .map_err(|e| format!("Failed to load source PDF '{}': {}", path, e))?;

         let id_offset = current_max_id; // Use the *current* max_id as the offset for this source doc
         let source_max_id = source_doc.max_id; // Remember the source's max ID

         // Perform the deep copy, ID shift, and reference updates
         // Map LopdfError to String here before propagating with ?
         let id_map = merge_and_shift_objects(&mut target_doc, &source_doc, id_offset, target_pages_id)
                .map_err(|e| format!("Failed merging objects from '{}': {}", path, e))?;

         // Update the max_id tracker for the *next* iteration
         current_max_id += source_max_id;

         // Collect the *new* references to the pages from the source doc
         let source_pages = source_doc.get_pages();
         for (_page_num, old_page_id) in source_pages {
              if let Some(new_page_id) = id_map.get(&old_page_id) {
                   new_kids_data.push(Object::Reference(*new_page_id));
              } else {
                   // This indicates an internal error if a page wasn't mapped
                   eprintln!("Warning: Page {:?} from '{}' was not found in the merged object map. Skipping.", old_page_id, path);
                   // Return error instead? Stricter:
                   return Err(format!("Internal error: Page {:?} from '{}' was not found in the merged object map.", old_page_id, path));
              }
         }
    } // End loop over paths

    // --- Finalize: Update Kids Array, Page Count, and Document Max ID ---
    // Update the target document's max_id *once* after all merging is done
    target_doc.max_id = current_max_id;

    { // Scope the final mutable borrow of the Pages dictionary
        let pages_dict_mut = target_doc
            .get_object_mut(target_pages_id)
            .map_err(|e| {
                format!(
                    "Failed to get mutable Pages object {:?} for final update: {}",
                    target_pages_id, e
                )
            })?
            .as_dict_mut()
            .map_err(|_| {
                format!(
                    "Final Pages object {:?} is not a dictionary",
                    target_pages_id
                )
            })?;

        // Get the mutable kids array
        let kids_array = pages_dict_mut
            .get_mut(b"Kids")
            .map_err(|e| format!("Failed to get mutable Kids entry for final update: {}", e))?
            .as_array_mut()
            .map_err(|_| "Final Pages Kids object is not an array".to_string())?;

        // Extend with the collected new page references (no clone needed)
        kids_array.extend(new_kids_data);

        // Calculate and set the final page count
        // Use the length of the *final* kids_array to be certain
        let final_page_count = kids_array.len() as i64;
        pages_dict_mut.set("Count", Object::Integer(final_page_count));

    } // End final mutable borrow

    // --- Save Merged Document ---
    target_doc.compress(); // Optional
    target_doc.save(output_path)
         .map_err(|e| format!("Failed to save merged PDF to '{}': {}", output_path, e))?;

    Ok(())
}


// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{dictionary, Document, Object, Stream}; // Ensure Dictionary is imported
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf}; // Use PathBuf
    use std::sync::atomic::{AtomicUsize, Ordering}; // For unique IDs

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

            // Use the dictionary! macro consistently
            let page_dict = dictionary! {
                "Type" => "Page",
                "Parent" => Object::Reference(pages_id),
                "MediaBox" => Object::Array(vec![0.into(), 0.into(), 612.into(), 792.into()]),
                "Contents" => Object::Reference(content_id),
                "Resources" => Object::Reference(resources_id),
            };
            // Pass the dictionary directly to add_object
            let page_id = doc.add_object(Object::Dictionary(page_dict)); // Corrected: wrap in Object::Dictionary
            kids.push(Object::Reference(page_id));
        }

        // Use dictionary! macro consistently
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => Object::Array(kids),
                // Correct way to convert u32 to Object::Integer
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

    // --- Use unique directory names ---
    // Base names
    const BASE_TEST_DIR: &str = "target/test_data_merge"; // Put test artifacts in target/
    const BASE_OUTPUT_DIR: &str = "target/test_output_merge";

    // Counter for unique test run IDs
    static TEST_RUN_ID: AtomicUsize = AtomicUsize::new(0);

    // Helper to get unique paths for a test run
    fn get_unique_paths(test_name: &str) -> (PathBuf, PathBuf) {
        let run_id = TEST_RUN_ID.fetch_add(1, Ordering::SeqCst);
        let unique_suffix = format!("{}_{}", test_name, run_id);

        let test_dir = PathBuf::from(BASE_TEST_DIR).join(&unique_suffix);
        let output_dir = PathBuf::from(BASE_OUTPUT_DIR).join(&unique_suffix);

        // Cleanup previous runs *of this specific test* (optional but good practice)
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).ok();
        }
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir).ok();
        }

        // Create fresh dirs for this run
        fs::create_dir_all(&test_dir).expect("Failed to create unique test data directory");
        fs::create_dir_all(&output_dir).expect("Failed to create unique test output directory");

        (test_dir, output_dir)
    }

    // Teardown helper (optional, target/ is usually cleaned by `cargo clean`)
    fn teardown_unique_paths(test_dir: &Path, output_dir: &Path) {
        fs::remove_dir_all(test_dir).ok();
        fs::remove_dir_all(output_dir).ok();
    }

    #[test]
    fn test_merge_two_pdfs_success() {
        let (test_dir, output_dir) = get_unique_paths("merge_two_pdfs"); // Get unique dirs

        let path1 = test_dir.join("doc1_2pages.pdf"); // Use unique test_dir
        let path2 = test_dir.join("doc2_1page.pdf"); // Use unique test_dir
        let output_path = output_dir.join("merged_2_1.pdf"); // Use unique output_dir

        create_minimal_pdf(path1.to_str().unwrap(), 2, "Doc1").expect("Create doc1");
        // Add a check to be absolutely sure
        assert!(path1.exists(), "doc1 should exist after creation");

        create_minimal_pdf(path2.to_str().unwrap(), 1, "Doc2").expect("Create doc2");
        assert!(path2.exists(), "doc2 should exist after creation");

        let paths_vec = vec![path1.to_str().unwrap(), path2.to_str().unwrap()];
        let result = merge_pdfs(paths_vec, output_path.to_str().unwrap());

        // Assertions remain the same
        assert!(result.is_ok(), "merge_pdfs failed: {:?}", result.err());
        assert!(output_path.exists(), "Output file was not created");
        match Document::load(output_path.to_str().unwrap()) {
            Ok(merged_doc) => {
                assert_eq!(
                    merged_doc.get_pages().len(),
                    3,
                    "Merged PDF should have 3 pages"
                );
                let catalog = merged_doc.catalog().expect("Merged catalog error");
                let pages_ref = catalog
                    .get(b"Pages")
                    .expect("Pages entry")
                    .as_reference()
                    .expect("Pages ref");
                let pages_dict = merged_doc
                    .get_dictionary(pages_ref)
                    .expect("Merged pages dict");
                let count = pages_dict
                    .get(b"Count")
                    .ok()
                    .and_then(|o| o.as_i64().ok())
                    .expect("Count");
                assert_eq!(count, 3, "Pages Count field mismatch");
            }
            Err(e) => panic!("Failed to load merged PDF: {}", e),
        }

        // Optional cleanup
        teardown_unique_paths(&test_dir, &output_dir);
    }

    #[test]
    fn test_merge_three_pdfs_success() {
        let (test_dir, output_dir) = get_unique_paths("merge_three_pdfs");

        let path1 = test_dir.join("d1.pdf");
        let path2 = test_dir.join("d2.pdf");
        let path3 = test_dir.join("d3.pdf");
        let output_path = output_dir.join("merged_1_2_3.pdf");

        create_minimal_pdf(path1.to_str().unwrap(), 1, "D1").expect("Create d1");
        create_minimal_pdf(path2.to_str().unwrap(), 2, "D2").expect("Create d2");
        create_minimal_pdf(path3.to_str().unwrap(), 3, "D3").expect("Create d3");
        assert!(path1.exists()); assert!(path2.exists()); assert!(path3.exists());


        let paths_vec = vec![path1.to_str().unwrap(), path2.to_str().unwrap(), path3.to_str().unwrap()];
        let result = merge_pdfs(paths_vec, output_path.to_str().unwrap());

        assert!(result.is_ok(), "merge_pdfs failed: {:?}", result.err());
        assert!(output_path.exists(), "Output file was not created");
        match Document::load(output_path.to_str().unwrap()) {
            Ok(merged_doc) => {
                assert_eq!(merged_doc.get_pages().len(), 6);
                // ... (verify count as before) ...
            }
            Err(e) => panic!("Failed to load merged PDF: {}", e),
        }
        teardown_unique_paths(&test_dir, &output_dir);
    }

    #[test]
    fn test_merge_input_not_a_pdf() {
        let (test_dir, output_dir) = get_unique_paths("test_merge_input_not_a_pdf");
        let path1 = test_dir.join("real.pdf");
        let not_pdf_path = test_dir.join("fake.txt");
        let output_path = output_dir.join("merged_fake_input.pdf");
        create_minimal_pdf(path1.to_str().unwrap(), 1, "Real").expect("Create real doc");
        // Corrected line:
        let mut file = fs::File::create(&not_pdf_path).expect("Failed to create dummy text file");
        writeln!(file, "This is text, not PDF.").expect("Failed to write to text file");
        let paths_vec = vec![path1.to_str().unwrap(), not_pdf_path.to_str().unwrap()];
        let result = merge_pdfs(paths_vec, output_path.to_str().unwrap());
        assert!(result.is_err());
        let err_msg = result.err().unwrap();
        assert!(err_msg.contains("Failed to load source PDF"));
        assert!(err_msg.contains(not_pdf_path.to_str().unwrap()));
        assert!(!output_path.exists());
        teardown_unique_paths(&test_dir, &output_dir);
    }

    #[test]
    fn test_merge_output_dir_not_found() {
        let (test_dir, output_dir) = get_unique_paths("test_merge_output_dir_not_found");
        fs::remove_dir_all(&output_dir).ok();
        let bad_output_dir = output_dir.join("subdir");
        let output_path = bad_output_dir.join("merged_bad_output.pdf");

        let path1 = test_dir.join("out_test1.pdf");
        let path2 = test_dir.join("out_test2.pdf");
        create_minimal_pdf(path1.to_str().unwrap(), 1, "Out1").expect("Failed to create out1");
        create_minimal_pdf(path2.to_str().unwrap(), 1, "Out2").expect("Failed to create out2");

        let paths_vec = vec![path1.to_str().unwrap(), path2.to_str().unwrap()];
        let result = merge_pdfs(paths_vec, output_path.to_str().unwrap());

        assert!(
            result.is_ok(),
            "merge failed creating dir: {:?}",
            result.err()
        );
        assert!(output_path.exists());

        match Document::load(output_path.to_str().unwrap()) {
            Ok(merged_doc) => assert_eq!(merged_doc.get_pages().len(), 2),
            Err(e) => panic!("Failed to load PDF from created dir: {}", e),
        }
        teardown_unique_paths(&test_dir, &output_dir);
    }
}