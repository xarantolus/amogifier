mod utils;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Uint8Array;
use web_sys::File;
use web_sys::FileReader;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, sus!");
}


#[wasm_bindgen]
pub async fn read_file_to_bytes(file: File) -> Result<Uint8Array, JsValue> {
    let file_reader = Rc::new(RefCell::new(FileReader::new().unwrap()));
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader_clone = file_reader.clone();
        let resolve_clone = resolve.clone();
        let reject_clone = reject.clone();

        let onloadend = Closure::wrap(Box::new(move || {
            let reader = reader_clone.borrow();
            if reader.ready_state() == FileReader::DONE {
                let array_buffer = reader.result().unwrap();
                let uint8_array = Uint8Array::new(&array_buffer);
                resolve_clone.call1(&JsValue::NULL, &uint8_array).unwrap();
            } else {
                reject_clone.call1(&JsValue::NULL, &JsValue::from_str("Failed to read file")).unwrap();
            }
        }) as Box<dyn FnMut()>);

        file_reader.borrow().set_onloadend(Some(onloadend.as_ref().unchecked_ref()));
        onloadend.forget();

        file_reader.borrow().read_as_array_buffer(&file).unwrap();
    });

    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result.dyn_into::<Uint8Array>()?)
}
