pub mod commands;
pub mod fs_wrapper;

use std::io;
use std::io::{Read, Write};
use std::path::Path;
use crate::commands::{built_in_commands, runner_for_built_in_command};
use anyhow::Result;
use clap::{ArgMatches, Command, crate_authors, crate_version};
use crate::fs_wrapper::{FakeFileSystem, FileSystemWrapper, ReadWrite};

const PROGRAM_NAME: &str = "ion";

#[macro_export]
macro_rules! print_to {
        ($out:ident, $($arg:tt)*) => {{
            $out.write(format!($($arg)*).as_bytes())?;
        }};
    }
#[macro_export]
macro_rules! println_to {
        ($out:ident) => {
            $out.write("\n".as_bytes())?;
        };
        ($out:ident, $($arg:tt)*) => {{
            $out.write(format!($($arg)*).as_bytes())?;
            $out.write(format!("\n").as_bytes())?;
        }};
    }


#[test]
pub fn test_lib() {
    let mut in_ = "{a:1, b:2}[".as_bytes();
    let mut out: Vec<u8> = Vec::new();

    let mut fake_fs = fs_wrapper::FakeFileSystem::new();

    let result = execute(|app| app.get_matches_from(vec!["ion","dump"]), &mut in_.clone(), &mut out, &mut fake_fs);

    if let Err(err) = result {
        println!("Err: {}", err)
    }

    println!("In: {}", String::from_utf8(in_.to_vec()).unwrap());
    println!("Out: {}", String::from_utf8(out).unwrap());
}


pub fn execute<'a, ArgFn: FnOnce(Command) -> ArgMatches, R: io::Read, W: io::Write, FS: crate::FileSystemWrapper<'a>>(arg_fn: ArgFn, in_: &mut R, out: &mut W, fs: &mut FS) -> Result<()> {

    let mut fake_fs = FakeFileSystem::new();
    let mut fake_file = fake_fs.create(&"foo")?;
    fake_file.write("abc".as_bytes())?;

    let mut app = Command::new(PROGRAM_NAME)
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand_required(true);

    for command in built_in_commands() {
        app = app.subcommand(command);
    }

    let args = (arg_fn)(app);
    let (command_name, command_args) = args.subcommand().unwrap();

    if let Some(runner) = runner_for_built_in_command(command_name) {
        // If a runner is registered for the given command name, command_args is guaranteed to
        // be defined.
        runner(command_name, command_args, in_, out, fs)?;
    } else {
        let message = format!(
            "The requested command ('{}') is not supported and clap did not generate an error message.",
            command_name
        );
        unreachable!("{}", message);
    }
    Ok(())
}
