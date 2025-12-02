#[test]
fn main() {
    use pdfium_render::prelude::*;

    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| {
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
                    "/usr/local/lib/",
                ))
            })
            .or_else(|_| Pdfium::bind_to_system_library())
            .unwrap(),
    );

    let docs = sdo_tests::docs_path();
    let img = docs.join("img");

    let path = img.join("PHYSIK.pdf");
    let doc = pdfium.load_pdf_from_file(&path, None).unwrap();

    let meta = doc.metadata();
    let creator = meta
        .get(PdfDocumentMetadataTagType::Creator)
        .expect("Creator");
    assert_eq!(creator.value(), "SIGNUM © 1986-93 F. Schmerbeck");

    let creator = meta
        .get(PdfDocumentMetadataTagType::Producer)
        .expect("Producer");
    assert_eq!(creator.value(), sdo_tests::PRODUCER);

    let render_config: PdfRenderConfig = PdfRenderConfig::new().set_target_height(4212);
    let image = doc
        .pages()
        .first()
        .expect("at least one page")
        .render_with_config(&render_config)
        .expect("render")
        .as_image();

    let save = img.join("physik-pdfium.png");
    image.save(save).unwrap();
}
