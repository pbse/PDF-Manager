use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

#[tauri::command]
pub fn markdown_to_pdf(title: String, content: String, output_path: String) -> Result<(), String> {
    let (doc, page1, layer1) = PdfDocument::new(&title, Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Load a professional font
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;

    // Header
    current_layer.use_text(&title, 24.0, Mm(20.0), Mm(270.0), &font);

    let font_body = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    
    // Simple line-based layout for content
    let mut y = 250.0;
    for line in content.lines() {
        if y < 20.0 { break; } // Limit to one page for now
        current_layer.use_text(line, 12.0, Mm(20.0), Mm(y), &font_body);
        y -= 7.0;
    }

    let file = File::create(&output_path).map_err(|e| e.to_string())?;
    doc.save(&mut BufWriter::new(file)).map_err(|e| e.to_string())?;

    Ok(())
}
