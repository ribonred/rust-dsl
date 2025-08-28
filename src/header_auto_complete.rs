use crate::{import::Import, style::set_styles};
use strum::VariantNames;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Element, HtmlElement};

const OVERLAY_ID: &str = "wasm-import-overlay";
const ACTIVE_ATTR: &str = "data-active";

pub fn overlay_active() -> bool {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(OVERLAY_ID) {
            if let Some(html) = el.dyn_ref::<HtmlElement>() {
                return html
                    .style()
                    .get_property_value("display")
                    .ok()
                    .map(|v| v != "none")
                    .unwrap_or(true);
            }
        }
    }
    false
}

pub fn attach_header_autocomplete(header: &HtmlElement) {
    let closure = Closure::wrap(Box::new(|event: web_sys::KeyboardEvent| {
        header_key_handler(event);
    }) as Box<dyn FnMut(_)>);
    header
        .add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())
        .ok();
    closure.forget();
}

fn header_key_handler(event: web_sys::KeyboardEvent) {
    if event.key() == "Escape" {
        hide_overlay(&event);
        return;
    }
    // Do not rebuild overlay on keyup after Tab cycling or Enter acceptance
    if event.key() == "Tab" || event.key() == "Enter" {
        return;
    }
    if let Some(target) = event.current_target() {
        if let Some(header_el) = target.dyn_ref::<HtmlElement>() {
            let code_text = header_el
                .first_child()
                .and_then(|ln| ln.next_sibling())
                .and_then(|n| {
                    n.dyn_ref::<HtmlElement>()
                        .map(|h| h.text_content().unwrap_or_default())
                })
                .unwrap_or_default();
            if code_text.starts_with('@') {
                show_overlay(&header_el, &code_text[1..]);
            } else {
                hide_existing_overlay(&header_el);
            }
        }
    }
}

fn show_overlay(header_line: &HtmlElement, filter: &str) {
    let doc = match header_line.owner_document() {
        Some(d) => d,
        None => return,
    };
    let window = match doc.default_view() {
        Some(w) => w,
        None => return,
    };
    let code_span = header_line
        .first_child()
        .and_then(|ln| ln.next_sibling())
        .and_then(|n| n.dyn_ref::<HtmlElement>().map(|h| h.clone()));
    let rect = code_span.as_ref().map(|c| c.get_bounding_client_rect());
    // Pre-compute matches
    let matches: Vec<&'static str> = Import::VARIANTS
        .iter()
        .filter(|k| k.to_lowercase().starts_with(&filter.to_lowercase()))
        .copied()
        .collect();
    let lower_filter = filter.to_lowercase();
    let exact_match = Import::VARIANTS
        .iter()
        .any(|k| k.to_lowercase() == lower_filter);
    if matches.is_empty() {
        if let Some(ex) = doc.get_element_by_id(OVERLAY_ID) {
            if let Some(html) = ex.dyn_ref::<HtmlElement>() {
                html.style().set_property("display", "none").ok();
            }
        }
        if let Some(code) = &code_span {
            apply_code_span_style(code, exact_match, true);
        }
        return;
    } else if let Some(code) = &code_span {
        apply_code_span_style(code, exact_match, false);
    }
    let mut overlay = doc.get_element_by_id(OVERLAY_ID);
    if overlay.is_none() {
        if let Ok(el) = doc.create_element("div") {
            overlay = Some(el);
        }
    }
    if let Some(ov) = overlay {
        ov.set_id(OVERLAY_ID);
        set_styles(
            &ov,
            &[
                ("position", "fixed"),
                ("z-index", "9999"),
                ("background", "#1e1e1e"),
                ("color", "#dcdcdc"),
                ("font-family", "monospace"),
                ("font-size", "12px"),
                ("border", "1px solid #444"),
                ("padding", "4px"),
                ("box-shadow", "0 4px 12px rgba(0,0,0,.4)"),
                ("max-height", "180px"),
                ("overflow", "auto"),
                ("min-width", "140px"),
                ("border-radius", "4px"),
                ("display", "block"),
            ],
        );
        update_overlay_items_with_matches(&ov, &matches);
        if let Some(r) = rect {
            let top = r.bottom() + 4.0;
            let mut left = r.left();
            let vw = window
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let estimated_width = 200.0;
            if left + estimated_width + 8.0 > vw {
                left = (vw - estimated_width - 8.0).max(0.0);
            }
            if let Some(html_el) = ov.dyn_ref::<HtmlElement>() {
                let style = html_el.style();
                style.set_property("top", &format!("{}px", top)).ok();
                style.set_property("left", &format!("{}px", left)).ok();
            }
        }
        if ov.parent_node().is_none() {
            if let Some(body) = doc.body() {
                let _ = body.append_child(&ov);
            }
        }
    }
}

fn update_overlay_items_with_matches(overlay: &Element, matches: &[&'static str]) {
    while let Some(child) = overlay.first_child() {
        let _ = overlay.remove_child(&child);
    }
    let mut first = true;
    for kw in matches.iter() {
        let value = (*kw).to_string();
        if let Some(doc) = overlay.owner_document() {
            if let Ok(item) = doc.create_element("div") {
                item.set_class_name("item");
                item.set_attribute("data-value", &value).ok();
                set_styles(
                    &item,
                    &[
                        ("padding", "2px 6px"),
                        ("cursor", "pointer"),
                        ("white-space", "nowrap"),
                        ("border-radius", "3px"),
                    ],
                );
                item.set_text_content(Some(&value));
                if first {
                    item.set_attribute(ACTIVE_ATTR, "true").ok();
                    first = false;
                    highlight_item(&item);
                }
                let _ = overlay.append_child(&item);
                let value_for_closure = value.clone();
                let closure = Closure::wrap(Box::new(move |ev: web_sys::Event| {
                    if let Some(t) = ev.target() {
                        if let Some(el) = t.dyn_ref::<HtmlElement>() {
                            if el.class_name() == "item" {
                                insert_selection(&value_for_closure);
                            }
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                item.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                    .ok();
                closure.forget();
            }
        }
    }
}

fn highlight_item(item: &Element) {
    set_styles(item, &[("background", "#264f78")]);
}

fn clear_highlights(overlay: &Element) {
    let mut maybe = overlay.first_element_child();
    while let Some(el) = maybe.clone() {
        el.remove_attribute(ACTIVE_ATTR).ok();
        if let Some(html_el) = el.dyn_ref::<HtmlElement>() {
            html_el.style().remove_property("background").ok();
        }
        maybe = el.next_element_sibling();
    }
}

pub fn cycle_overlay(next: bool) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(overlay) = doc.get_element_by_id(OVERLAY_ID) {
            if overlay
                .get_attribute("style")
                .map(|s| s.contains("display:none"))
                .unwrap_or(false)
            {
                return;
            }
            // collect items
            let mut items: Vec<Element> = Vec::new();
            let mut maybe = overlay.first_element_child();
            while let Some(el) = maybe.clone() {
                items.push(el.clone());
                maybe = el.next_element_sibling();
            }
            if items.is_empty() {
                return;
            }
            let mut idx: isize = items
                .iter()
                .position(|e| e.get_attribute(ACTIVE_ATTR).is_some())
                .map(|i| i as isize)
                .unwrap_or(-1);
            if next {
                idx += 1;
            } else {
                idx -= 1;
            }
            if idx < 0 {
                idx = items.len() as isize - 1;
            }
            if idx as usize >= items.len() {
                idx = 0;
            }
            clear_highlights(&overlay);
            if let Some(sel) = items.get(idx as usize) {
                sel.set_attribute(ACTIVE_ATTR, "true").ok();
                highlight_item(sel);
            }
        }
    }
}

pub fn accept_overlay_selection() {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(overlay) = doc.get_element_by_id(OVERLAY_ID) {
            if overlay
                .get_attribute("style")
                .map(|s| s.contains("display:none"))
                .unwrap_or(false)
            {
                return;
            }
            let mut maybe = overlay.first_element_child();
            while let Some(el) = maybe.clone() {
                if el.get_attribute(ACTIVE_ATTR).is_some() {
                    if let Some(val) = el.get_attribute("data-value") {
                        insert_selection(&val);
                    }
                    break;
                }
                maybe = el.next_element_sibling();
            }
        }
    }
}

fn hide_overlay(event: &web_sys::KeyboardEvent) {
    let doc_opt = event
        .target()
        .and_then(|t| t.dyn_ref::<HtmlElement>().map(|h| h.owner_document()))
        .flatten();
    if let Some(doc) = doc_opt {
        if let Some(el) = doc.get_element_by_id(OVERLAY_ID) {
            if let Some(html) = el.dyn_ref::<HtmlElement>() {
                html.style().set_property("display", "none").ok();
            }
        }
    }
}

fn hide_existing_overlay(header_line: &HtmlElement) {
    if let Some(doc) = header_line.owner_document() {
        if let Some(el) = doc.get_element_by_id(OVERLAY_ID) {
            if let Some(html) = el.dyn_ref::<HtmlElement>() {
                html.style().set_property("display", "none").ok();
            }
        }
    }
}

fn insert_selection(selected: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Some(selection) = window.get_selection().ok().flatten() {
                if let Some(anchor) = selection.anchor_node() {
                    if let Some(code_span) = find_code_span(anchor) {
                        code_span.set_text_content(Some(&format!("@{}", selected)));
                        selection.remove_all_ranges().ok();
                        apply_code_span_style(&code_span, true, false);
                    }
                }
            }
            if let Some(el) = doc.get_element_by_id(OVERLAY_ID) {
                if let Some(html) = el.dyn_ref::<HtmlElement>() {
                    html.style().set_property("display", "none").ok();
                }
            }
        }
    }
}

fn find_code_span(start: web_sys::Node) -> Option<HtmlElement> {
    let mut current: Option<web_sys::Node> = Some(start);
    while let Some(node) = current {
        if let Some(el) = node.dyn_ref::<HtmlElement>() {
            if el.class_name() == "code" {
                return Some(el.clone());
            }
        }
        current = node.parent_node();
    }
    None
}

fn apply_code_span_style(code: &HtmlElement, exact: bool, no_matches: bool) {
    if let Some(html) = code.dyn_ref::<HtmlElement>() {
        let st = html.style();
        if no_matches {
            st.set_property("text-decoration", "underline").ok();
            st.set_property("text-decoration-color", "#ff4d4f").ok();
            st.set_property("text-decoration-thickness", "0.5px").ok();
            st.set_property("text-decoration-style", "wavy").ok();
            st.remove_property("color").ok();
        } else {
            st.remove_property("text-decoration").ok();
            st.remove_property("text-decoration-color").ok();
            st.remove_property("text-decoration-thickness").ok();
            st.remove_property("text-decoration-style").ok();
            if exact {
                st.set_property("color", "#1754edff").ok();
            } else {
                st.remove_property("color").ok();
            }
        }
    }
}
