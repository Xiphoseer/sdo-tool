use mupdf::{Colorspace, ImageFormat, Matrix};

#[test]
fn render_mupdf() {
    let docs = sdo_tests::docs_path();
    let img = docs.join("img");

    let path = img.join("PHYSIK.pdf");
    let file = std::fs::read(path).expect("file");
    let pdf = mupdf::pdf::PdfDocument::from_bytes(&file).expect("from_bytes");
    let first_page = pdf.load_page(0).expect("load_page");
    let ctm = &Matrix::new_scale(5.0, 5.0); // 72 * 5 = 360 dpi
    let cs = &Colorspace::device_gray();
    let alpha = false;
    let show_extras = false;
    let pixmap = first_page
        .to_pixmap(ctm, cs, alpha, show_extras)
        .expect("to_picmap");

    let output = img.join("physik-mupdf.png");
    let filename = output.to_string_lossy();
    let format = ImageFormat::PNG;
    pixmap.save_as(filename.as_ref(), format).expect("save_as");
}
