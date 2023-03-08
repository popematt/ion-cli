use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use ion_rs::*;
use std::io::{self, BufReader};


pub fn app() -> Command {
    Command::new("count")
        .about("Prints the number of top-level values found in the input stream.")
        .arg(
            // All argv entries after the program name (argv[0])
            // and any `clap`-managed options are considered input files.
            Arg::new("input")
                .index(1)
                .help("Input file [default: STDIN]")
                .action(ArgAction::Append)
                .trailing_var_arg(true),
        )
}

pub fn run<'a, R: io::Read, W: io::Write, FS: crate::FileSystemWrapper<'a>>(_command_name: &str, matches: &ArgMatches, in_: &mut R, out: &mut W, fs: &mut FS) -> Result<()> {    if let Some(input_file_iter) = matches.get_many::<String>("input") {
        for input_file in input_file_iter {
            let file = fs.open(input_file)
                .with_context(|| format!("Could not open file '{}'", input_file))?;
            let mut reader = ReaderBuilder::new().build(file)?;
            print_top_level_value_count(&mut reader)?;
        }
    } else {
        let buf_reader = BufReader::new(in_);
        let mut reader = ReaderBuilder::new().build(buf_reader)?;
        print_top_level_value_count(&mut reader)?;
    };

    Ok(())
}

fn print_top_level_value_count(reader: &mut Reader) -> Result<()> {
    let mut count: usize = 0;
    loop {
        let item = reader
            .next()
            .with_context(|| "could not count values in Ion stream")?;
        if item == StreamItem::Nothing {
            break;
        }
        count += 1;
    }
    println!("{}", count);
    Ok(())
}
