use std::io;
use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use ion_rs::binary::var_int::VarInt;
use ion_rs::binary::var_uint::VarUInt;
use crate::{println_to, print_to};

pub fn app() -> Command {
    Command::new("primitive")
        .about("Prints the binary representation of an Ion encoding primitive.")
        .arg(
            Arg::new("type")
                .short('t')
                .required(true)
                .help("The Ion primitive encoding type. (Names are case insensitive.)")
                .value_parser(["VarInt", "varint", "VarUInt", "varuint"]),
        )
        .arg(
            Arg::new("value")
                .short('v')
                .required(true)
                .allow_hyphen_values(true)
                .help("The value to encode as the specified primitive."),
        )
}

pub fn run<'a, R: io::Read, W: io::Write, FS: crate::FileSystemWrapper<'a>>(_command_name: &str, matches: &ArgMatches, in_: &mut R, out: &mut W, fs: &mut FS) -> anyhow::Result<()> {

    let mut buffer = Vec::new();
    let value_text = matches.get_one::<String>("value").unwrap().as_str();
    match matches.get_one::<String>("type").unwrap().as_str() {
        "varuint" | "VarUInt" => {
            let value = integer_from_text(value_text)? as u64;
            VarUInt::write_u64(&mut buffer, value).unwrap();
        }
        "varint" | "VarInt" => {
            let value = integer_from_text(value_text)?;
            VarInt::write_i64(&mut buffer, value).unwrap();
        }
        unsupported => {
            unreachable!(
                "clap did not reject unsupported primitive encoding {}",
                unsupported
            );
        }
    }
    print_to!(out, "hex: ");
    for byte in buffer.iter() {
        // We want the hex bytes to align with the binary bytes that will be printed on the next
        // line. Print 6 spaces and a 2-byte hex representation of the byte.
        print_to!(out, "      {:0>2x} ", byte);
    }
    println_to!(out);
    print_to!(out, "bin: ");
    for byte in buffer.iter() {
        // Print the binary representation of each byte
        print_to!(out, "{:0>8b} ", byte);
    }
    println_to!(out);
    Ok(())
}

fn integer_from_text(text: &str) -> Result<i64> {
    if text.starts_with("0x") {
        i64::from_str_radix(text, 16)
            .with_context(|| format!("{} is not a valid hexidecimal integer value.", text))
    } else if text.starts_with("0b") {
        i64::from_str_radix(text, 2)
            .with_context(|| format!("{} is not a valid binary integer value.", text))
    } else {
        text.parse::<i64>()
            .with_context(|| format!("{} is not a valid decimal integer value.", text))
    }
}
