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
            Ok(paths
                .into_iter()
                .filter_map(|file_path| {
                    // Use filter_map to handle potential non-Path variants
                    if let FilePath::Path(p) = file_path {
                        Some(p.to_string_lossy().into_owned())
                    } else {
                        eprintln!("Received non-path FilePath variant: {:?}", file_path);
                        None // Skip non-path variants if they occur
                    }
                })
                .collect())
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
                eprintln!(
                    "Received non-path FilePath variant on save: {:?}",
                    file_path
                );
                Ok(None) // Treat unexpected variant as cancellation/failure
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

// --- IMPORTANT: Update main.rs ---
/*
// Example for your src-tauri/src/main.rs
use tauri::Manager; // Needed for AppHandle methods if used in setup

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        // .plugin(tauri_plugin_shell::init()) // Can likely remove if only using opener
        .plugin(tauri_plugin_opener::init()) // Add opener
        .setup(|app| {
            // Ensure window exists for dialogs/shell operations potentially needing it
            let _window = app.get_window("main").unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Commands are async again for dialogs/shell_open
            commands::open_file_dialog,
            commands::save_file_dialog,
            commands::get_os_type,
            commands::shell_open,
            // Your PDF commands
            // pdf::parse_pdf,
            // pdf::merge_pdfs,
            // pdf::split_pdf,
            // pdf::extract_pdf_page
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
*/

// --- IMPORTANT: Update Cargo.toml ---
/*
# src-tauri/Cargo.toml
[dependencies]
# ... other dependencies ...
tauri = { version = "2.0.0-beta.4", features = [] } # Adjust version as needed
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Add/Update Tauri Plugins
tauri-plugin-dialog = "2.0.0-beta.3"
tauri-plugin-os = "2.0.0-beta.2"
tauri-plugin-opener = "2.0.0-beta.2" # Add opener plugin crate
# tauri-plugin-shell = "2.0.0-beta.3" # Remove if not needed

# Add Tokio for async runtime features like oneshot channels
tokio = { version = "1", features = ["sync"] }

# Your PDF library
lopdf = "0.31.0"

[features]
# If using custom protocol for assets:
# custom-protocol = ["tauri/custom-protocol"]
default = []
*/
