#[cfg(test)]
use lopdf::{dictionary, Document, Object, Stream};

/// Create a simple PDF with a shared font/resource and N pages containing text.
#[cfg(test)]
pub fn create_minimal_pdf(
    file_path: &str,
    num_pages: u32,
    text_prefix: &str,
) -> std::io::Result<()> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! { "F1" => Object::Reference(font_id) },
        "ProcSet" => Object::Array(vec![
            Object::Name(b"PDF".to_vec()),
            Object::Name(b"Text".to_vec()),
        ]),
    });

    let mut kids = Vec::new();
    for i in 1..=num_pages {
        let content_str =
            format!("BT /F1 12 Tf 100 700 Td ({}-Page {}) Tj ET", text_prefix, i);
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

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    doc.save(file_path)?;
    Ok(())
}
