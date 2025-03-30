#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

// Declare the 'pdf' module (Rust needs this to find pdf/mod.rs)
mod commands;
mod pdf;

// Use the re-exported functions directly from the 'pdf' module
use crate::pdf::{extract_pdf_page, merge_pdfs, parse_pdf, split_pdf};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init()) // Needs the shell plugin for v2
        .plugin(tauri_plugin_dialog::init()) // Needs the dialog plugin for v2
        .plugin(tauri_plugin_os::init()) // Needs the os plugin for v2 (if you use more os features)
        .plugin(tauri_plugin_opener::init()) // Ensure opener is initialized
        .setup(|app| {
            // It's often better to handle potential errors rather than unwrap
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(debug_assertions)]
                {
                    // Only open devtools if debug assertions are enabled
                    window.open_devtools();
                    // window.close_devtools(); // Optionally close them if opened automatically
                }
            } else {
                eprintln!("Error: Could not get main window");
                // Handle the error appropriately, maybe exit or log
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            parse_pdf,
            merge_pdfs,
            split_pdf,
            extract_pdf_page,
            commands::open_file_dialog,
            commands::save_file_dialog,
            commands::get_os_type,
            commands::shell_open,
            // Ensure these names match exactly what's imported above
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
