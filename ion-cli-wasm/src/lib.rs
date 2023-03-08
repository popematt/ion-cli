use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::format;
use std::io::Read;
use std::rc::Rc;
use std::str;
use ion_rs::IonType;
use wasm_bindgen::prelude::*;
use ion_cli_core;
use ion_cli_core::execute;
use ion_rs::value::{Builder, IonElement, IonSequence, IonStruct};
use ion_rs::value::owned::{Element, Struct, Value};
use ion_rs::value::reader::element_reader;
use ion_rs::value::reader::ElementReader;
use ion_cli_core::fs_wrapper::FakeFileSystem;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type IonCliInput = { "command": [string], "stdin": string, };
export type IonCliOutput = { "stdout": string, "error", string? };

// export function execute_command(input: IonCliInput): IonCliOutput;
"#;

const _JS_APPEND_CONTENT: &'static str = r#"
import {execute_command_raw } from "./ion_cli_wasm";

export function execute_command(input) {
    return JSON.parse(execute_command_raw(JSON.stringify(input)));
}
"#;


#[wasm_bindgen]
pub struct FsHandle {
    file_system: Rc<RefCell<HashMap<String, Vec<u8>>>>
}
impl FsHandle {
    fn new() -> Self {
        FsHandle {
            file_system: Rc::new(RefCell::new(HashMap::new()))
        }
    }
}

#[wasm_bindgen]
pub fn create_fs() -> FsHandle {
    FsHandle::new()
}

#[wasm_bindgen]
pub fn fs_add_file(handle: FsHandle, name: String, bytes: Vec<u8>) {
    RefCell::borrow_mut(&handle.file_system).insert(name, bytes);
}

#[wasm_bindgen]
pub fn fs_read_file(handle: FsHandle, name: String) -> Vec<u8> {
    match RefCell::borrow_mut(&handle.file_system).get(&*name) {
        None => vec![],
        Some(vec) => vec.clone(),
    }
}

#[wasm_bindgen]
pub fn destroy_fs(handle: FsHandle) {
    RefCell::borrow_mut(&handle.file_system).clear()
}

/// Executes an ion-cli command.
///
#[wasm_bindgen]
pub fn execute_command_raw(input: &str) -> String {
    let input_element = element_reader().read_one(input.as_bytes()).unwrap();
    let input_element = input_element.as_struct().unwrap();

    eprintln!("Start with input: {}", input);

    let mut in_ = input_element.get("stdin").unwrap().as_str().unwrap().as_bytes();
    let mut out: Vec<u8> = Vec::new();

    let arg_list = input_element.get("command").unwrap().as_sequence()
        .unwrap()
        .iter()
        .filter_map(|it| it.as_str());

    let mut fake_file_system = FakeFileSystem::new();

    let result = execute(|app| app.get_matches_from(arg_list), &mut in_, &mut out, &mut fake_file_system);

    format!(
        r#" {{ "stdout": {}, "error": {} }} "#,
        Element::from(String::from_utf8(out).unwrap()),
        if let Err(e) = result {
            Element::from(format!("{}", e)).to_string()
        } else {
            "null".to_string()
        },
    )
}

#[test]
fn test_ion_dump() {
    let input = r#"
        {
          "command": ["ion", "dump"],
          "stdin": "foo::(bar [1, 2, 3] baz)"
        }
    "#;

    let output = execute_command_raw(input);

    println!("Output: {}", output);
}

#[test]
fn test_ion_beta_count() {
    let input = r#"
        {
          "command": ["ion", "beta", "count"],
          "stdin": "foo::(bar [1, 2, 3] baz) bar baz"
        }
    "#;

    let output = execute_command_raw(input);

    println!("Output: {}", output);
}

#[test]
fn test_ion_beta_inspect() {
    let input = r#"
        {
          "command": ["ion", "beta", "inspect", "'foo::(bar [1, 2, 3] baz)'"],
          "stdin": ""
        }
    "#;

    let output = execute_command_raw(input);

    println!("Output: {}", output);
}