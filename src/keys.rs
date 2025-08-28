use crate::layout::renumber_lines;
use crate::line_handlers::{create_line, get_headers};
use crate::log;
use crate::header_auto_complete::{cycle_overlay, accept_overlay_selection, overlay_active};
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{self, HtmlElement};

pub fn attach_key_handler(container: &web_sys::HtmlElement) {
    let container_clone = container.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let key = event.key();
        // If overlay visible, intercept Tab (cycle) and Enter (accept)
        if overlay_active() {
            if key == "Tab" { event.prevent_default(); cycle_overlay(true); return; }
            if key == "Enter" { event.prevent_default(); accept_overlay_selection(); return; }
        }
        if event.alt_key() && (key == "x" || key == "X") {
            handle_alt_x(&container_clone);
            return;
        }
        match key.as_str() {
            "Enter" => handle_enter(&event),
            "Backspace" => handle_backspace(&event, &container_clone),
            "ArrowUp" => handle_arrow(&event, Direction::Up),
            "ArrowDown" => handle_arrow(&event, Direction::Down),
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);
    container
        .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .ok();
    closure.forget();
}

fn handle_alt_x(container: &HtmlElement) {
    if let Some(header) = get_headers(container) {
        // Log the actual element to the browser console (rich expandable log)
        web_sys::console::log_1(&header.into());
        log::tagged("ALT+X", "Logged header element");
    } else {
        log::tagged("ALT+X", "Header element not found");
    }
}

fn handle_enter(event: &web_sys::KeyboardEvent) {
    event.prevent_default();
    if let Some(target) = event.target() {
        if let Some(code_el) = target.dyn_ref::<web_sys::Element>() {
            if code_el.class_name() == "code" {
                if let Some(line_el) = code_el.parent_element() {
                    if let Some(next_line) = line_el.next_element_sibling() {
                        if let Some(code) = next_line.last_child() {
                            if let Some(code_html) = code.dyn_ref::<web_sys::HtmlElement>() {
                                let _ = code_html.focus();
                            }
                        }
                    } else {
                        let doc = line_el.owner_document().unwrap();
                        if let Some(new_line) = create_line(&doc, 0) {
                            if let Some(parent) = line_el.parent_element() {
                                let _ = parent.append_child(&new_line);
                                renumber_lines(&parent.dyn_ref::<web_sys::HtmlElement>().unwrap());
                                if let Some(code) = new_line.last_child() {
                                    if let Some(code_html) = code.dyn_ref::<web_sys::HtmlElement>()
                                    {
                                        let _ = code_html.focus();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn handle_backspace(event: &web_sys::KeyboardEvent, container: &HtmlElement) {
    if let Some(target) = event.target() {
        if let Some(code_el) = target.dyn_ref::<web_sys::HtmlElement>() {
            if code_el.class_name() == "code" {
                let text = code_el.text_content().unwrap_or_default();
                if text.is_empty() {
                    if let Some(line_el) = code_el.parent_element() {
                        if line_el.get_attribute("data-line").as_deref() == Some("1") {
                            event.prevent_default();
                        } else {
                            event.prevent_default();
                            if let Some(prev) = line_el.previous_element_sibling() {
                                if let Some(parent) = line_el.parent_element() {
                                    let _ = parent.remove_child(&line_el);
                                }
                                renumber_lines(container);
                                if let Some(code) = prev.last_child() {
                                    if let Some(code_html) = code.dyn_ref::<web_sys::HtmlElement>()
                                    {
                                        let _ = code_html.focus();
                                        if let Some(doc) = code_html.owner_document() {
                                            if let Ok(range) = doc.create_range() {
                                                range.select_node_contents(&code_html).ok();
                                                let _ = range.collapse_with_to_start(false);
                                                if let Some(sel) = doc
                                                    .default_view()
                                                    .and_then(|w| w.get_selection().ok())
                                                    .flatten()
                                                {
                                                    sel.remove_all_ranges().ok();
                                                    sel.add_range(&range).ok();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

enum Direction {
    Up,
    Down,
}
fn handle_arrow(event: &web_sys::KeyboardEvent, dir: Direction) {
    if let Some(target) = event.target() {
        if let Some(code_el) = target.dyn_ref::<HtmlElement>() {
            if code_el.class_name() == "code" {
                if let Some(line_el) = code_el.parent_element() {
                    let sibling = match dir {
                        Direction::Up => line_el.previous_element_sibling(),
                        Direction::Down => line_el.next_element_sibling(),
                    };
                    if let Some(sib) = sibling {
                        if let Some(code) = sib.last_child() {
                            if let Some(code_html) = code.dyn_ref::<HtmlElement>() {
                                event.prevent_default();
                                let _ = code_html.focus();
                                if let Some(doc) = code_html.owner_document() {
                                    if let Ok(range) = doc.create_range() {
                                        range.select_node_contents(&code_html).ok();
                                        let _ = range.collapse_with_to_start(false);
                                        if let Some(sel) = doc
                                            .default_view()
                                            .and_then(|w| w.get_selection().ok())
                                            .flatten()
                                        {
                                            sel.remove_all_ranges().ok();
                                            sel.add_range(&range).ok();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
