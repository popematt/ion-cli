use ion_rs::*;
use std::env;
use struct_with_fields::StructWithFields;

include!(concat!(env!("OUT_DIR"), "/ion_data_model/struct_with_fields.rs"));


#[test]
fn test_roundtrip_generated_code_rust() {
    let input_dir = format!("{}/input", env!("CARGO_MANIFEST_DIR"));

    let foo = StructWithFields::new(1, "foo".to_string(), true);
    println!("{:?}", foo);
    // TODO: Add ser/de round trip
}

