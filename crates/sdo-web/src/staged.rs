use wasm_bindgen::JsValue;

use crate::Handle;

impl Handle {
    pub(super) async fn show_sdoc3(&self, name: &str, _data: &[u8]) -> Result<(), JsValue> {
        let heading = self.document.create_element("h2")?;
        heading.set_text_content(Some(name));
        self.output.append_child(&heading)?;
        let p = self.document.create_element("p")?;
        p.append_with_str_1("Signum! 3/4 Document")?;
        self.output.append_child(&p)?;
        Ok(())
    }

    pub(super) async fn show_cset2(&self, name: &str, _data: &[u8]) -> Result<(), JsValue> {
        let heading = self.document.create_element("h2")?;
        heading.set_text_content(Some(name));
        self.output.append_child(&heading)?;
        let p = self.document.create_element("p")?;
        p.append_with_str_1("Signum! 3/4 Font")?;
        self.output.append_child(&p)?;
        Ok(())
    }
}
