use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Thread-local storage for closures (WASM is single-threaded)
thread_local! {
    static CLOSURES: std::cell::RefCell<std::collections::HashMap<i32,Closure<dyn Fn()>>>  = std::cell::RefCell::new(std::collections::HashMap::new());
}

// Get the global scope (works in both Window and Worker)
pub fn get_global() -> Result<js_sys::Object, JsValue> {
    js_sys::Reflect::get(&JsValue::from(js_sys::global()), &"self".into()).map(|v| v.into())
}

// Abstract interface for set_interval that works in both contexts
pub fn set_interval_universal(callback: &js_sys::Function, timeout: i32) -> Result<i32, JsValue> {
    let global = js_sys::global();

    // Try to call setInterval on the global object
    let set_interval = js_sys::Reflect::get(&global, &"setInterval".into())?;
    let set_interval_fn: js_sys::Function = set_interval.dyn_into()?;

    let args = js_sys::Array::new();
    args.push(callback);
    args.push(&JsValue::from(timeout));

    let result = set_interval_fn.apply(&global, &args)?;
    Ok(result.as_f64().unwrap() as i32)
}

pub fn clear_interval_universal(interval_id: i32) -> Result<(), JsValue> {
    let global = js_sys::global();

    let clear_interval = js_sys::Reflect::get(&global, &"clearInterval".into())?;
    let clear_interval_fn: js_sys::Function = clear_interval.dyn_into()?;

    let args = js_sys::Array::new();
    args.push(&JsValue::from(interval_id));

    clear_interval_fn.apply(&global, &args)?;
    Ok(())
}

// Detect if we're in a Worker or Window context
pub fn detect_context() -> &'static str {
    let global = js_sys::global();

    // Check for WorkerGlobalScope
    if js_sys::Reflect::has(&global, &"WorkerGlobalScope".into()).unwrap_or(false) {
        return "worker";
    }

    // Check for Window
    if js_sys::Reflect::has(&global, &"Window".into()).unwrap_or(false) {
        return "window";
    }

    // Fallback: check for document (only available in Window)
    if js_sys::Reflect::has(&global, &"document".into()).unwrap_or(false) {
        return "window";
    }

    "unknown"
}

/// Run a Rust function at a specified interval
///
/// # Arguments
/// * `func` - The Rust function to run (must be 'static and thread-safe)
/// * `interval_ms` - Interval in milliseconds
///
/// # Returns
/// * `i32` - The interval ID that can be used to stop the interval
#[wasm_bindgen]
pub fn start_interval_with_fn(interval_ms: i32, func: &js_sys::Function) -> Result<i32, JsValue> {
    let interval_id = set_interval_universal(func, interval_ms)?;
    Ok(interval_id)
}

/// Create an interval that runs a Rust closure
/// This is a lower-level function for use from Rust code
pub fn create_interval<F>(interval_ms: i32, func: F) -> Result<i32, JsValue>
where
    F: Fn() + 'static,
{
    let closure = Closure::wrap(Box::new(func) as Box<dyn Fn()>);

    let callback: &js_sys::Function = closure.as_ref().unchecked_ref();
    let interval_id = set_interval_universal(callback, interval_ms)?;

    // Store closure to keep it alive
    CLOSURES.with(|closures| {
        closures.borrow_mut().insert(interval_id, closure);
    });

    Ok(interval_id)
}

/// Create an interval that runs a Rust closure with mutable state
/// This is for closures that need to capture and mutate variables
pub fn create_interval_mut<F>(interval_ms: i32, func: F) -> Result<i32, JsValue>
where
    F: FnMut() + 'static,
{
    let closure = Closure::wrap(Box::new(func) as Box<dyn FnMut()>);

    let callback: &js_sys::Function = closure.as_ref().unchecked_ref();
    let interval_id = set_interval_universal(callback, interval_ms)?;

    // Store closure to keep it alive (transmute for storage purposes)
    CLOSURES.with(|closures| {
        closures.borrow_mut().insert(interval_id, unsafe {
            std::mem::transmute::<Closure<dyn FnMut()>, Closure<dyn Fn()>>(closure)
        });
    });

    Ok(interval_id)
}

#[wasm_bindgen]
pub fn stop_interval(interval_id: i32) -> Result<(), JsValue> {
    // Remove and drop the closure
    CLOSURES.with(|closures| {
        closures.borrow_mut().remove(&interval_id);
    });
    clear_interval_universal(interval_id)
}

#[wasm_bindgen]
pub fn get_context_info() -> String {
    detect_context().to_string()
}
