use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

mod parser;
use parser::{parse_first_line, ParsedFirstLine};

thread_local! {
    static BUFFER: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
}

#[wasm_bindgen]
pub struct Stats {
    pub len: usize,
    pub lines: usize,
    pub valid: bool,
    message: String,
}

#[wasm_bindgen]
impl Stats {
    #[wasm_bindgen(constructor)]
    pub fn new(len: usize, lines: usize, valid: bool, message: String) -> Stats {
        Stats {
            len,
            lines,
            valid,
            message,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
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
    textarea
        .set_attribute(
            "style",
            "flex:1;min-height:60vh;font-family:inherit;font-size:14px;line-height:1.4;tab-size:4;white-space:pre;",
        )
        .unwrap();
    let textarea_el: web_sys::Element = textarea.clone().into();

    // Autocomplete popup (hidden initially)
    let ac = document.create_element("div").unwrap();
    ac.set_attribute("id", "autocomplete").ok();
    ac.set_attribute("style", "position:absolute;z-index:10;background:white;border:1px solid #ccc;font-size:12px;font-family:inherit;display:none;max-width:240px;box-shadow:0 2px 6px rgba(0,0,0,0.15);").unwrap();
    body.append_child(&ac).unwrap();

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

    // Closure for input event (stats + autocomplete)
    let stats_clone = stats.clone();
    let textarea_for_closure = textarea.clone();
    let ac_for_closure = ac.clone();

    let closure = Closure::wrap(Box::new(move || {
        let val = textarea_for_closure.value();
        let s = apply_change(&val);
        if s.valid {
            stats_clone.set_text_content(Some(&format!("Length: {}\nLines: {}", s.len, s.lines)));
        } else {
            stats_clone.set_text_content(Some(&format!(
                "Length: {}\nLines: {}\nError: {}",
                s.len, s.lines, s.message
            )));
        }
        // Autocomplete: only active in first line before any newline
        let cursor = textarea_for_closure
            .selection_start()
            .ok()
            .flatten()
            .unwrap_or(0) as usize;
        if let Some(first_nl_pos) = val.find('\n') {
            if cursor > first_nl_pos { ac_for_closure.set_attribute("style", "display:none").ok(); return; }
        }
        // Extract current token after '@'
        let prefix_opt = val[..cursor].rfind('@').map(|idx| (idx, &val[idx+1..cursor]));
        let directives = ["option", "multipleoption", "matchingpair"];
        if let Some((at_pos, frag)) = prefix_opt {
            // Ensure '@' is at start or preceded by whitespace
            let valid_start = at_pos == 0 || val.as_bytes()[at_pos-1].is_ascii_whitespace();
            if !valid_start { ac_for_closure.set_attribute("style", "display:none").ok(); }
            else {
                let matches: Vec<&str> = directives.iter().copied().filter(|d| d.starts_with(frag)).collect();
                if matches.is_empty() { ac_for_closure.set_attribute("style", "display:none").ok(); }
                else {
                    // Build list HTML
                    let html = matches.iter().enumerate().map(|(i,m)| format!("<div data-item='{m}' style='padding:2px 6px;cursor:pointer;{}'>{}</div>", if i==0 {"background:#eef;"} else {""}, m)).collect::<String>();
                    let mut style = String::from("position:absolute;z-index:10;background:white;border:1px solid #ccc;font-size:12px;font-family:inherit;max-width:240px;box-shadow:0 2px 6px rgba(0,0,0,0.15);");
                    // Rough position: align under textarea top-left; (simple, not caret position)
                    style.push_str("display:block;top:10px;left:10px;padding:2px 0;");
                    ac_for_closure.set_attribute("style", &style).ok();
                    ac_for_closure.set_inner_html(&html);
                }
            }
        } else {
            ac_for_closure.set_attribute("style", "display:none").ok();
        }
    }) as Box<dyn FnMut()>);

    textarea
        .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
        .unwrap();
    // Keep the closure alive (leak for simplicity; could store to drop later)
    closure.forget();

    // Key handler for accepting first suggestion with Tab or Enter
    let textarea_key = textarea.clone();
    let ac_key = ac.clone();
    let key_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        if event.key() == "Tab" || event.key() == "Enter" {
            let html = ac_key.inner_html();
            if html.is_empty() { return; }
            // Extract first data-item
            if let Some(start) = html.find("data-item='") {
                if let Some(rest) = html[start+11..].split_once("'") {
                    let item = rest.0;
                    let cursor = textarea_key
                        .selection_start()
                        .ok()
                        .flatten()
                        .unwrap_or(0) as usize;
                    let mut val = textarea_key.value();
                    let at_pos = val[..cursor].rfind('@').unwrap_or(cursor);
                    // Replace fragment with full item
                    val.replace_range(at_pos+1..cursor, item);
                    textarea_key.set_value(&val);
                    // Move cursor to end of inserted item
                    let new_cursor = at_pos + 1 + item.len();
                    textarea_key.set_selection_start(Some(new_cursor as u32)).ok();
                    textarea_key.set_selection_end(Some(new_cursor as u32)).ok();
                    ac_key.set_attribute("style", "display:none").ok();
                    apply_change(&textarea_key.value()); // update buffer
                    event.prevent_default();
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    textarea
        .add_event_listener_with_callback("keydown", key_closure.as_ref().unchecked_ref())
        .unwrap();
    key_closure.forget();

    // Initialize stats with initial content
    let init_stats = apply_change(&textarea.value());
    stats.set_text_content(Some(&if init_stats.valid {
        format!("Length: {}\nLines: {}", init_stats.len, init_stats.lines)
    } else {
        format!(
            "Length: {}\nLines: {}\nError: {}",
            init_stats.len, init_stats.lines, init_stats.message
        )
    }));
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
        let (valid, message) = match parse_first_line(&buf) {
            Ok(ParsedFirstLine::Empty) => (true, String::new()),
            Ok(_) => (true, String::new()),
            Err(e) => (false, e),
        };
        Stats::new(buf.len(), lines, valid, message)
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
