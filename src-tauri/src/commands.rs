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
    // Command is async again
    app_handle: AppHandle,
    multiple: bool,
) -> Result<Vec<String>, String> {
    // Return type remains the same

    // Create a one-shot channel to receive the result from the callback
    let (tx, rx) = oneshot::channel::<Option<Vec<FilePath>>>();

    let dialog = app_handle.dialog().file(); // Get the dialog builder

    if multiple {
        dialog
            .add_filter("PDF Files", &["pdf"])
            .set_title("Select PDF File(s)")
            // The callback now sends the result through the channel sender 'tx'
            .pick_files(move |paths: Option<Vec<FilePath>>| {
                // Send the result. Ignore error if receiver (rx) was dropped.
                let _ = tx.send(paths);
            });
    } else {
        dialog
            .add_filter("PDF Files", &["pdf"])
            .set_title("Select PDF File")
            // The callback now sends the result through the channel sender 'tx'
            .pick_file(move |path: Option<FilePath>| {
                // Convert Option<FilePath> to Option<Vec<FilePath>> before sending
                let result_vec = path.map(|p| vec![p]);
                let _ = tx.send(result_vec);
            });
    }

    // Asynchronously wait for the result from the channel receiver 'rx'
    match rx.await {
        Ok(Some(paths)) => {
            // Callback sent Some(Vec<FilePath>)
            // Process the Vec<FilePath>
            paths
                .into_iter()
                .map(|file_path| {
                    if let FilePath::Path(p) = file_path {
                        Ok(p.to_string_lossy().into_owned())
                    } else {
                        Err("Received non-path FilePath variant".to_string())
                    }
                })
                .collect::<Result<Vec<String>, String>>()
        }
        Ok(None) => Ok(vec![]), // Callback sent None (User cancelled)
        Err(e) => Err(format!("Failed to receive dialog result: {}", e)), // Error receiving from channel
    }
}

#[command]
pub async fn save_file_dialog(
    // Command is async again
    app_handle: AppHandle,
    default_path: String,
) -> Result<Option<String>, String> {
    // Return Option<String>

    // Create a one-shot channel
    let (tx, rx) = oneshot::channel::<Option<FilePath>>();

    app_handle
        .dialog()
        .file()
        .add_filter("PDF Files", &["pdf"])
        .set_title("Save PDF File")
        .set_file_name(&default_path)
        // Callback sends result via channel
        .save_file(move |path: Option<FilePath>| {
            let _ = tx.send(path);
        });

    // Asynchronously wait for the result
    match rx.await {
        Ok(Some(file_path)) => {
            // Callback sent Some(FilePath)
            // Extract PathBuf from FilePath
            if let FilePath::Path(p) = file_path {
                Ok(Some(p.to_string_lossy().into_owned()))
            } else {
                Err("Received non-path FilePath variant on save".to_string())
            }
        }
        Ok(None) => Ok(None), // Callback sent None (User cancelled)
        Err(e) => Err(format!("Failed to receive save dialog result: {}", e)), // Channel error
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