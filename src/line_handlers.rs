use crate::header_auto_complete::attach_header_autocomplete;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

pub fn create_line(doc: &web_sys::Document, number: usize) -> Option<Element> {
    let line = doc.create_element("div").ok()?;
    line.set_class_name("wasm-line");
    line.set_attribute("data-line", &number.to_string()).ok();
    let ln = doc.create_element("span").ok()?;
    ln.set_class_name("ln");
    ln.set_text_content(Some(&number.to_string()));
    let code = doc.create_element("span").ok()?;
    code.set_class_name("code");
    code.set_attribute("contenteditable", "true").ok();
    let _ = line.append_child(&ln);
    let _ = line.append_child(&code);
    Some(line)
}

pub fn get_headers(container: &HtmlElement) -> Option<HtmlElement> {
    let mut maybe = container.first_element_child();
    while let Some(el) = maybe.clone() {
        if el
            .dyn_ref::<Element>()
            .map(|e| e.class_name() == "wasm-line")
            .unwrap_or(false)
        {
            if el.get_attribute("data-line").as_deref() == Some("1") {
                return el.dyn_ref::<HtmlElement>().map(|h| h.clone());
            }
        }
        maybe = el.next_element_sibling();
    }
    None
}

pub fn header_handler(container: &HtmlElement) {
    if let Some(header) = get_headers(&container) {
        attach_header_autocomplete(&header);
    }
}
