use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(s: &str);
}

/// Log an info message to the browser console.
pub fn info(message: impl AsRef<str>) { log(message.as_ref()); }

/// Log an error message to the browser console.
pub fn error(message: impl AsRef<str>) { console_error(message.as_ref()); }

/// Log with a prefix tag to help grouping.
pub fn tagged(tag: &str, message: impl AsRef<str>) { log(&format!("[{}] {}", tag, message.as_ref())); }
