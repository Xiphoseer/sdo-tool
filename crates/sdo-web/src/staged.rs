use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlAnchorElement;

use crate::Handle;

impl Handle {
    pub(super) async fn show_sdoc3(&self, name: &str, _data: &[u8]) -> Result<(), JsValue> {
        let heading = self.document.create_element("h2")?;
        heading.set_text_content(Some(name));
        self.output.append_child(&heading)?;
        let p = self.document.create_element("p")?;
        p.append_with_str_1("Signum! 3/4 Document")?;
        let br = self.document.create_element("br")?;
        p.append_child(&br)?;
        let a = self.document.create_element("a")?;
        a.set_text_content(Some("Not yet implemented"));
        let a = a.dyn_ref::<HtmlAnchorElement>().unwrap();
        a.set_href("https://github.com/Xiphoseer/sdo-tool/issues/19");
        a.set_target("_blank");
        p.append_child(a)?;
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
