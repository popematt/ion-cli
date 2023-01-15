mod commands;

// pub use crate::commands;

use std::io;
use crate::commands::{built_in_commands, runner_for_built_in_command};
use anyhow::Result;
use clap::{Command, crate_authors, crate_version};

const PROGRAM_NAME: &str = "ion";

#[test]
pub fn test_it() {
    let mut in_ = "".as_bytes();
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();

    let in_out = Io {
        in_: Box::new(&mut in_),
        out: Box::new(&mut out),
        err: Box::new(&mut err),
    };
    
    execute(vec!["ion","dump","--help"], in_out).expect("TODO: panic message");

    println!("In: {}", String::from_utf8(in_.to_vec()).unwrap());
    println!("Out: {}", String::from_utf8(out).unwrap());
    println!("Err: {}", String::from_utf8(err).unwrap());
}

pub struct Io<'a> {
    in_: Box<&'a mut dyn io::Read>,
    out: Box<&'a mut dyn io::Write>,
    err: Box<&'a mut dyn io::Write>,
}

pub struct Io2<'a> {
    in_: &'a mut dyn io::Read,
    out: &'a mut dyn io::Write,
    err: &'a mut dyn io::Write,
}

fn foo2(io: &Io2) {
    io.out.write_fmt(format_args!("foo")).expect("");

}


pub fn execute(cmd: Vec<&str>, in_out: Io) -> Result<()> {
    
    let mut app = create_app();

    let args = app.get_matches_from(cmd);
    let (command_name, command_args) = args.subcommand().unwrap();

    if let Some(runner) = runner_for_built_in_command(command_name) {
        // If a runner is registered for the given command name, command_args is guaranteed to
        // be defined.
        runner(command_name, command_args, in_out)?;
    } else {
        let message = format!(
            "The requested command ('{}') is not supported and clap did not generate an error message.",
            command_name
        );
        unreachable!("{}", message);
    }
    Ok(())
}

pub fn main() -> Result<()> {

    let mut in_ = io::stdin().lock();
    let mut out = io::stdout().lock();
    let mut err = io::stderr().lock();

    let mut in_: Box<&mut dyn io::Read> = Box::new(&mut in_);
    let mut out: Box<&mut dyn io::Write> = Box::new(&mut out);
    let mut err: Box<&mut dyn io::Write> = Box::new(&mut err);


    let in_out = Io {
        in_,
        out,
        err,
    };

    let app = create_app();

    let args = app.get_matches();
    let (command_name, command_args) = args.subcommand().unwrap();

    if let Some(runner) = runner_for_built_in_command(command_name) {
        // If a runner is registered for the given command name, command_args is guaranteed to
        // be defined.
        runner(command_name, command_args, in_out)?;
    } else {
        let message = format!(
            "The requested command ('{}') is not supported and clap did not generate an error message.",
            command_name
        );
        unreachable!("{}", message);
    }
    Ok(())
}

fn create_app() -> Command {

    let mut app = Command::new(PROGRAM_NAME)
        .version(crate_version!())
        .author(crate_authors!())
        .subcommand_required(true);

    for command in built_in_commands() {
        app = app.subcommand(command);
    }

    app
}