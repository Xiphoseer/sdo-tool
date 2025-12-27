use std::sync::Arc;

use hayro::{InterpreterSettings, Pdf, RenderSettings, render};

#[test]
fn render_hayro() {
    let docs = sdo_tests::docs_path();
    let img = docs.join("img");

    let path = img.join("PHYSIK.pdf");

    let file = std::fs::read(path).unwrap();
    let scale = 5.0;

    let data = Arc::new(file);
    let pdf = Pdf::new(data).unwrap();

    let interpreter_settings = InterpreterSettings::default();

    let render_settings = RenderSettings {
        x_scale: scale,
        y_scale: scale,
        ..Default::default()
    };

    let page = pdf.pages().first().unwrap();
    let pixmap = render(page, &interpreter_settings, &render_settings);
    let save = img.join("physik-hayro.png");
    std::fs::write(save, pixmap.take_png()).unwrap();
}
