use wasm_bindgen::JsCast;

/// Inject base stylesheet for the editor (idempotent)
pub(crate) fn inject_base_styles(document: &web_sys::Document) {
    if document
        .get_element_by_id("wasm-editor-base-style")
        .is_some()
    {
        return;
    }
    if let Ok(style) = document.create_element("style") {
        style.set_id("wasm-editor-base-style");
        style.set_text_content(Some(r#"/* Basic editor styles */
#editor-wasm { position:relative; font-family:monospace; font-size:14px; line-height:1.4; background:#fff; border:1px solid #ccc; min-height:160px; outline:none; }
#editor-wasm .wasm-line { display:flex; align-items:flex-start; }
#editor-wasm .wasm-line .ln { width:3em; text-align:right; padding-right:.5em; color:#888; user-select:none; }
#editor-wasm .wasm-line .code { flex:1; min-height:1.4em; white-space:pre-wrap; outline:none; }
#editor-wasm .wasm-line .code:focus { background:#f5faff; }
"#));
        if let Ok(Some(head)) = document.query_selector("head") {
            let _ = head.append_child(&style);
        } else if let Some(body) = document.body() {
            let _ = body.append_child(&style);
        }
    }
}

/// Renumber existing line elements inside the container.
pub(crate) fn renumber_lines(container: &web_sys::HtmlElement) {
    let mut index = 1usize;
    let mut maybe_child = container.first_element_child();
    while let Some(child) = maybe_child {
        if child
            .dyn_ref::<web_sys::Element>()
            .map(|e| e.class_name() == "wasm-line")
            .unwrap_or(false)
        {
            child.set_attribute("data-line", &index.to_string()).ok();
            if let Some(first) = child.first_child() {
                if let Some(span) = first.dyn_ref::<web_sys::Element>() {
                    span.set_text_content(Some(&index.to_string()));
                }
            }
        }
        index += 1;
        maybe_child = child.next_element_sibling();
    }
}

/// Populate editor with initial empty lines if empty.
pub(crate) fn init(
    container: &web_sys::HtmlElement,
    lines: usize,
    create_line: impl Fn(usize) -> Option<web_sys::Element>,
) {
    if container.first_element_child().is_some() {
        return;
    }
    for i in 1..=lines {
        if let Some(line) = create_line(i) {
            let _ = container.append_child(&line);
        }
    }
    if let Some(first) = container.first_element_child() {
        if let Some(code) = first.last_child() {
            if let Some(code_el) = code.dyn_ref::<web_sys::HtmlElement>() {
                let _ = code_el.focus();
            }
        }
    }
}
