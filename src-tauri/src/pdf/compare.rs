use std::collections::HashSet;

#[derive(serde::Serialize)]
pub struct ComparisonResult {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub shared_count: usize,
}

#[tauri::command]
pub fn compare_pdfs_text(path1: &str, path2: &str) -> Result<ComparisonResult, String> {
    let text1 = pdf_extract::extract_text(path1).map_err(|e| e.to_string())?;
    let text2 = pdf_extract::extract_text(path2).map_err(|e| e.to_string())?;

    let lines1: HashSet<&str> = text1.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let lines2: HashSet<&str> = text2.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    let added: Vec<String> = lines2.difference(&lines1).map(|s| s.to_string()).collect();
    let removed: Vec<String> = lines1.difference(&lines2).map(|s| s.to_string()).collect();
    let shared_count = lines1.intersection(&lines2).count();

    Ok(ComparisonResult { added, removed, shared_count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::test_utils::{create_minimal_pdf, setup_unique_paths, teardown_unique_paths};

        #[test]
    fn test_compare_pdfs_success() {
        let (test_dir, output_dir) = setup_unique_paths("compare_success");
        let path1 = test_dir.join("v1.pdf");
        let path2 = test_dir.join("v2.pdf");
        
        create_minimal_pdf(path1.to_str().unwrap(), 1, "Original").unwrap();
        create_minimal_pdf(path2.to_str().unwrap(), 1, "Modified").unwrap();
        
        let result = compare_pdfs_text(path1.to_str().unwrap(), path2.to_str().unwrap()).unwrap();
        
        // We expect "Modified-Page 1" to be in added and "Original-Page 1" to be in removed
        assert!(result.added.iter().any(|l| l.contains("Modified")));
        assert!(result.removed.iter().any(|l| l.contains("Original")));
        
        teardown_unique_paths(&test_dir, &output_dir);
    }

}
