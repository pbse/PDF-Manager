use tauri::{command, AppHandle};
use tokio::sync::oneshot; // Use oneshot channels for async callback results

// Import necessary Extension Traits and Types
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_opener::OpenerExt; // <-- Import the Opener extension trait

// No ShellExt needed if using opener
// use tauri_plugin_shell::ShellExt;

// --- Dialogs (Using Async Commands with Async Callbacks via Channels) ---

#[command]
pub async fn open_file_dialog(
    app_handle: AppHandle,
    multiple: bool,
) -> Result<Vec<String>, String> {
    let dialog = app_handle.dialog().file();

    let (tx, rx) = oneshot::channel();
    if multiple {
        dialog
            .add_filter("PDF Files", &["pdf"])
            .set_title("Select PDF File(s)")
            .pick_files(move |files| {
                let _ = tx.send(files);
            });
    } else {
        dialog
            .add_filter("PDF Files", &["pdf"])
            .set_title("Select PDF File")
            .pick_file(move |file| {
                let _ = tx.send(file.map(|fp| vec![fp])); // Convert single pick to vec
            });
    }

    let result = rx.await.map_err(|e| e.to_string())?;

    match result {
        Some(paths) => paths
            .into_iter()
            .map(|file_path| {
                if let FilePath::Path(p) = file_path {
                    Ok(p.to_string_lossy().into_owned())
                } else {
                    Err("Received non-path FilePath variant".to_string())
                }
            })
            .collect::<Result<Vec<String>, String>>(),
        None => Ok(vec![]), // User cancelled
    }
}

#[command]
pub async fn save_file_dialog(
    app_handle: AppHandle,
    default_path: String,
) -> Result<Option<String>, String> {
    let (tx, rx) = oneshot::channel();
    app_handle
        .dialog()
        .file()
        .add_filter("PDF Files", &["pdf"])
        .set_title("Save PDF File")
        .set_file_name(&default_path)
        .save_file(move |file| {
            let _ = tx.send(file);
        });

    let result = rx.await.map_err(|e| e.to_string())?;

    match result {
        Some(file_path) => {
            if let FilePath::Path(p) = file_path {
                Ok(Some(p.to_string_lossy().into_owned()))
            } else {
                Err("Received non-path FilePath variant on save".to_string())
            }
        }
        None => Ok(None), // User cancelled
    }
}

#[command]
pub fn get_os_type() -> Result<String, String> {
    // This command remains synchronous and unchanged
    Ok(std::env::consts::OS.to_string())
}

// --- Shell Open (Using tauri-plugin-opener) ---
#[command]
pub async fn shell_open(
    // Make async to use opener's async function if available
    app_handle: AppHandle,
    file_path: String,
) -> Result<(), String> {
    // Use the opener plugin's open function
    app_handle
        .opener()
        .open_url(file_path, None::<String>) // Call the open_url method directly on app_handle
        .map_err(|e| e.to_string()) // Map the opener::Error to String
}

#[command]
pub async fn reveal_in_folder(
    app_handle: AppHandle,
    file_path: String,
) -> Result<(), String> {
    app_handle
        .opener()
        .reveal_item_in_dir(file_path)
        .map_err(|e| e.to_string())
}

#[command]
pub fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("Failed to read file at {}: {}", path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_file_bytes_success() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Hello, World!").unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let result = read_file_bytes(path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"Hello, World!\n");
    }

    #[test]
    fn test_read_file_bytes_not_found() {
        let path = "/path/to/non_existent_file.pdf".to_string();
        let result = read_file_bytes(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }
}
