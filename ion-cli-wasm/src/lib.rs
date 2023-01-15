
use std::str;
use wasm_bindgen::prelude::*;
use ion_cli_core;

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct IonCliCommand(String);

#[wasm_bindgen]
pub struct IonCliOutput {
    std_out: String,
    std_err: String,
    exit_code: u8,
}

#[wasm_bindgen]
pub fn execute(cmd: String) -> IonCliOutput {

    todo!()
}
