#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

// Declare the 'pdf' module (Rust needs this to find pdf/mod.rs)
mod commands;
mod pdf;

// Use the re-exported functions directly from the 'pdf' module
use crate::pdf::{
    add_annotation, add_ink_annotation, delete_annotation, update_annotation_contents, replace_text_block, pdf_to_docx, add_signature_visual, delete_pages, extract_pdf_page, merge_pdfs, parse_pdf,
    rotate_pdf, sign_pdf_pfx, split_pdf, verify_signatures, sanitize_pdf, reorder_pages, compress_pdf, decrypt_pdf, encrypt_pdf, flatten_annotations, images_to_pdf, pdf_to_images, add_watermark, start_folder_watcher, forensic_redact, compare_pdfs_text, update_metadata, batch_update_metadata, get_pdf_outline, set_pdf_outline, get_annotations, get_form_fields, set_form_fields, create_form_fields, pdf_to_text, pdf_to_text_string, write_text_file, markdown_to_pdf, generate_briefing,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init()) // Needs the shell plugin for v2
        .plugin(tauri_plugin_dialog::init()) // Needs the dialog plugin for v2
        .plugin(tauri_plugin_os::init()) // Needs the os plugin for v2 (if you use more os features)
        .plugin(tauri_plugin_opener::init()) // Ensure opener is initialized
        .setup(|app| {
            // It's often better to handle potential errors rather than unwrap
            if let Some(_window) = app.get_webview_window("main") {
                #[cfg(debug_assertions)]
                {
                    // Only open devtools if debug assertions are enabled
                    _window.open_devtools();
                    // _window.close_devtools(); // Optionally close them if opened automatically
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
            rotate_pdf,
            delete_pages,
            add_annotation,
            add_ink_annotation,
            delete_annotation,
            update_annotation_contents,
            add_signature_visual,
            sign_pdf_pfx,
            verify_signatures,
            sanitize_pdf,
            reorder_pages,
            compress_pdf,
            decrypt_pdf,
            encrypt_pdf,
            flatten_annotations,
            images_to_pdf,
            pdf_to_images,
            add_watermark,
            start_folder_watcher,
 forensic_redact, markdown_to_pdf, generate_briefing, compare_pdfs_text, update_metadata, batch_update_metadata, get_pdf_outline, set_pdf_outline,            get_annotations,
            get_form_fields,
            set_form_fields,
            create_form_fields,
            replace_text_block,
            pdf_to_docx,
            pdf_to_text,
            pdf_to_text_string,
            write_text_file,


            commands::open_file_dialog,
            commands::save_file_dialog,
            commands::get_os_type,
            commands::shell_open,
            commands::reveal_in_folder,
            commands::read_text_file,
            commands::get_file_hash,
            commands::write_file_bytes,
            commands::read_file_bytes,
            commands::rename_file,
            // Ensure these names match exactly what's imported above
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
