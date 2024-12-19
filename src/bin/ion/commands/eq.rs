use crate::commands::IonCliCommand;
use crate::hex_reader::HexReader;
use crate::input::CommandInput;
use anyhow::Result;
use clap::error::ErrorKind::{TooFewValues, TooManyValues};
use clap::{Arg, ArgAction, ArgMatches, Command};
use ion_rs::*;
use std::fs::File;
use std::io;
use std::io::{Cursor, Read};

pub struct EqCommand;

impl IonCliCommand for EqCommand {
    fn name(&self) -> &'static str {
        "eq"
    }

    fn about(&self) -> &'static str {
        "Compares two Ion streams, returning a non-zero exit code if they are not equivalent."
    }

    fn is_stable(&self) -> bool {
        false
    }

    fn is_porcelain(&self) -> bool {
        false
    }

    fn configure_args(&self, command: Command) -> Command {
        command
            .after_help(
                "Exactly two Ion streams must be provided, and may be provided as a file name, a \
                string of hexadecimal pairs, or a string of Ion text. If only one stream is provided \
                as an argument or option, then stdin is implied to be the second Ion stream."
            )
            .arg(Arg::new("input")
                .action(ArgAction::Append)
                .help("File(s) containing an Ion stream"))
            .arg(Arg::new("hex-input")
                .long("hex")
                .action(ArgAction::Append)
                .help("An Ion binary stream that is provided as a string of hexadecimal pairs"))
            .arg(Arg::new("text-input")
                .long("text")
                .short('t')
                .action(ArgAction::Append)
                .help("An Ion text stream"))
            .arg(Arg::new("boolean-output")
                .help("Prints true or false to stdout.")
                .action(ArgAction::SetTrue)
                .short('B')
                .long("bool"))
    }

    fn run(&self, _command_path: &mut Vec<String>, args: &ArgMatches) -> Result<()> {
        let is_boolean_output = args.get_flag("boolean-output");
        let no_auto_decompression = args.get_flag("no-auto-decompress");

        let mut inputs_vec: Vec<Result<Sequence>> = vec![];

        if let Some(inputs) = args.get_many::<String>("input") {
            inputs.for_each(|it| inputs_vec.push(read_file_input(it, no_auto_decompression)));
        }
        if let Some(inputs) = args.get_many::<String>("hex-input") {
            inputs.for_each(|it| inputs_vec.push(read_hex_input(it)));
        }
        if let Some(inputs) = args.get_many::<String>("text-input") {
            inputs.for_each(|it| inputs_vec.push(read_text_input(it)));
        }

        match inputs_vec.len() {
            0 => self.clap_command().error(TooFewValues, "two Ion streams required; found 0").exit(),
            1 => inputs_vec.push(read_stdin_input(no_auto_decompression)),
            2 => {}
            n => self.clap_command().error(TooManyValues, format!("two Ion streams required; found {n}")).exit(),
        }

        // There are now exactly two elements in inputs_vec
        let mut inputs = inputs_vec.into_iter();
        let data1 = inputs.next().unwrap()?;
        let data2 = inputs.next().unwrap()?;
        let is_equivalent = IonData::eq(&data1, &data2);

        if is_boolean_output {
            println!("{}", is_equivalent);
        }
        if !is_equivalent {
            std::process::exit(1);
        }
        Ok(())
    }
}

fn read_file_input(file_name: &String, no_auto_decompression: bool) -> Result<Sequence> {
    let command_input = if no_auto_decompression {
        CommandInput::without_decompression(file_name, File::open(file_name)?)?
    } else {
        CommandInput::decompress(file_name, File::open(file_name)?)?
    };
    let data = command_input
        .into_source()
        .bytes()
        .collect::<Result<Vec<_>, io::Error>>()?;
    Ok(Element::read_all(data)?)
}

fn read_hex_input(hex_string: &String) -> Result<Sequence> {
    let hex_reader = HexReader::from(Cursor::new(hex_string));
    let bytes = hex_reader.bytes().collect::<Result<Vec<u8>, io::Error>>()?;
    Ok(Element::read_all(bytes)?)
}

fn read_text_input(text: &String) -> Result<Sequence> {
    Ok(Element::read_all(text)?)
}

fn read_stdin_input(no_auto_decompression: bool) -> Result<Sequence> {
    const STDIN_NAME: &str = "-";
    let stdin = io::stdin().lock();
    let command_input = if !no_auto_decompression {
        CommandInput::decompress(STDIN_NAME, stdin)
    } else {
        CommandInput::without_decompression(STDIN_NAME, stdin)
    }?;
    let data = command_input
        .into_source()
        .bytes()
        .collect::<Result<Vec<_>, io::Error>>()?;
    Ok(Element::read_all(data)?)
}
