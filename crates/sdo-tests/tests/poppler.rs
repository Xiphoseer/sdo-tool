use std::io::BufWriter;

use sdo_tests::PRODUCER;

#[test]
fn render_poppler() {
    let docs = sdo_tests::docs_path();
    let img = docs.join("img");

    let path = img.join("PHYSIK.pdf");
    let input_filename = format!("file://{}", path.display());
    let pdf = poppler::Document::from_file(&input_filename, None).expect("from_file");

    let creator = pdf.creator().expect("creator");
    assert_eq!(creator.as_str(), "SIGNUM © 1986-93 F. Schmerbeck");

    let producer = pdf.producer().expect("producer");
    assert_eq!(producer.as_str(), PRODUCER);

    let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, 2960, 4210)
        .expect("ImageSurface::create");
    let context = cairo::Context::new(&surface).expect("Context::new");

    context.set_source_rgb(1.0, 1.0, 1.0);
    context.rectangle(0.0, 0.0, 2960.0, 4210.0);
    context.fill().expect("fill");

    context.scale(5.0, 5.0);
    let first_page = pdf.page(0).unwrap();
    first_page.render(&context);

    let save = img.join("physik-poppler.png");
    let file = std::fs::File::create(save).expect("File::create");
    let mut writer = BufWriter::new(file);
    surface.write_to_png(&mut writer).expect("write_to_png");
}
