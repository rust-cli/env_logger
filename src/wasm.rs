//! logging functions from wasm-bindgen.
//!
//! Here use the one-param logging functions, all messages should be transformed
//! to string before passing to the functions.
#![cfg(target="wasm32-unknown-unknown")]

use log::Level;

fn format_message(record: &Record) -> String {
    format!("{<5}: {}", record.level(), record.args())
}

pub fn print(msg: &str, lv: Level) {
    match lv {
        Level::Error => err(&msg),
        Level::Warn => warn(&msg),
        Level::Info => info(&msg),
        Level::Debug => debug(&msg),
        Level::Trace => log(&msg),
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=console)]
    pub fn debug(text: &str);

    #[wasm_bindgen(js_namespace=console)]
    pub fn info(text: &str);

    #[wasm_bindgen(js_namespace=console)]
    pub fn log(text: &str);

    #[wasm_bindgen(js_namespace=console)]
    pub fn warn(text: &str);

    #[wasm_bindgen(js_namespace=console)]
    pub fn error(text: &str);
}
