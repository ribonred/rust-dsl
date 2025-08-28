use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

thread_local! {
    static BUFFER: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}

#[wasm_bindgen]
pub struct Stats {
    pub len: usize,
    pub lines: usize,
}

#[wasm_bindgen]
impl Stats {
    #[wasm_bindgen(constructor)]
    pub fn new(len: usize, lines: usize) -> Stats {
        Stats { len, lines }
    }
}

#[wasm_bindgen]
pub fn init_editor() {
    log("editor initialized");
    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");
    let body = document.body().expect("no body");

    let container = document.create_element("div").unwrap();
    container
        .set_attribute(
            "style",
            "display:flex;gap:1rem;align-items:stretch;font-family:monospace;",
        )
        .unwrap();
    let textarea = document
        .create_element("textarea")
        .unwrap()
        .dyn_into::<web_sys::HtmlTextAreaElement>()
        .unwrap();
    textarea.set_id("editor-container");
    textarea.set_spellcheck(false);
    textarea.set_value("# Start typing...\n");
    textarea
        .set_attribute(
            "style",
            "flex:1;min-height:60vh;font-family:inherit;font-size:14px;line-height:1.4;tab-size:4;white-space:pre;",
        )
        .unwrap();
    let textarea_el: web_sys::Element = textarea.clone().into();

    // Stats pre element
    let stats = document.create_element("pre").unwrap();
    stats
        .set_attribute(
            "style",
            "width:180px;margin:0;padding:0.5rem;font-size:12px;background:#f5f5f5;border:1px solid #ddd;overflow:auto;",
        )
        .unwrap();
    stats.set_text_content(Some("Length: 0\nLines: 0"));

    container.append_child(&textarea_el).unwrap();
    container.append_child(&stats).unwrap();
    body.append_child(&container).unwrap();

    // Closure for input event
    let stats_clone = stats.clone();
    let textarea_for_closure = textarea.clone();

    let closure = Closure::wrap(Box::new(move || {
        let val = textarea_for_closure.value();
        let s = apply_change(&val);
        stats_clone.set_text_content(Some(&format!("Length: {}\nLines: {}", s.len, s.lines)));
    }) as Box<dyn FnMut()>);

    textarea
        .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
        .unwrap();
    // Keep the closure alive (leak for simplicity; could store to drop later)
    closure.forget();

    // Initialize stats with initial content
    let init_stats = apply_change(&textarea.value());
    stats.set_text_content(Some(&format!(
        "Length: {}\nLines: {}",
        init_stats.len, init_stats.lines
    )));
}

#[wasm_bindgen]
pub fn apply_change(full_text: &str) -> Stats {
    BUFFER.with(|b| {
        let mut buf = b.borrow_mut();
        buf.clear();
        buf.push_str(full_text);
        let lines = if buf.is_empty() {
            0
        } else {
            buf.as_bytes().iter().filter(|&&c| c == b'\n').count() + 1
        };
        Stats::new(buf.len(), lines)
    })
}

// Optional helper callable from JS if needed to fetch the current buffer later.
#[wasm_bindgen]
pub fn get_buffer() -> String {
    BUFFER.with(|b| b.borrow().clone())
}

fn log(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}
