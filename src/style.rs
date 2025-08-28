use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

pub fn set_styles(el: &Element, styles: &[(&str, &str)]) {
    if let Some(html_el) = el.dyn_ref::<HtmlElement>() {
        let style = html_el.style();
        for (k, v) in styles {
            style.set_property(k, v).ok();
        }
    }
}

pub fn remove_style_props(el: &Element, props: &[&str]) {
    if let Some(html_el) = el.dyn_ref::<HtmlElement>() {
        let style = html_el.style();
        for p in props {
            style.remove_property(p).ok();
        }
    }
}
