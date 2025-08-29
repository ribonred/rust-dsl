use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
pub mod header_auto_complete;
pub mod import;
pub mod keys;
pub mod layout; // new module for layout & line population
pub mod line_handlers;
pub mod log;
pub mod parser;
pub mod style; // include parser module for native tests
pub use line_handlers::create_line;

use crate::layout::{init, inject_base_styles};
use crate::line_handlers::header_handler;

fn ensure_container() -> Option<web_sys::HtmlElement> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let el = document.get_element_by_id("editor-wasm")?;
    let html_el = el.dyn_ref::<web_sys::HtmlElement>()?.clone();
    inject_base_styles(&document);
    // Container itself not directly editable; child spans are.
    if html_el.get_attribute("contenteditable").is_some() {
        html_el.remove_attribute("contenteditable").ok();
    }
    // Assign class name (avoid classList for minimal feature usage)
    let existing = html_el.class_name();
    if existing.is_empty() {
        html_el.set_class_name("wasm-editor");
    } else if !existing.split_whitespace().any(|c| c == "wasm-editor") {
        html_el.set_class_name(&format!("{} wasm-editor", existing));
    }
    Some(html_el)
}

// (moved functions now live in layout.rs)

fn setup_key_events(container: &web_sys::HtmlElement) {
    keys::attach_key_handler(container);
}

#[wasm_bindgen]
pub fn init_editor() {
    if let Some(container) = ensure_container() {
        // supply creator closure expected by init
        init(&container, 5, |i| {
            if let Some(doc) = container.owner_document() {
                create_line(&doc, i)
            } else {
                None
            }
        });
        setup_key_events(&container);
        header_handler(&container);
    } else {
        log::info("No container found");
    }
}

#[wasm_bindgen]
pub fn noop() {}
