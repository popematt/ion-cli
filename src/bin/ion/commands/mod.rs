use crate::file_writer::FileWriter;
use crate::input::CommandInput;
use crate::output::CommandOutput;
use anyhow::Context;
use anyhow::Result;
use clap::builder::ValueParser;
use clap::{crate_authors, crate_version, Arg, ArgAction, ArgMatches, Command as ClapCommand};
use std::fs::File;
use std::io::Write;
use termcolor::{ColorChoice, StandardStream, StandardStreamLock};

pub mod cat;
mod command_namespace;
pub mod complaint;
pub mod from;
pub mod generate;
pub mod hash;
pub mod head;
pub mod inspect;
pub mod jq;
pub mod primitive;
pub mod schema;
pub mod stats;
pub mod symtab;
pub mod to;

pub(crate) use command_namespace::IonCliNamespace;

/// Behaviors common to all Ion CLI commands, including both namespaces (groups of commands)
/// and the commands themselves.
pub trait IonCliCommand {
    /// Indicates whether this command is stable (as opposed to unstable or experimental).
    /// Namespaces should almost always be stable.
    fn is_stable(&self) -> bool {
        false
    }

    /// Whether the output format is machine-readable.
    ///
    /// Commands that are "plumbing" should default to putting one output (result, value, document)
    /// on each line in a machine-readable format (file name, Ion value(s), integers, booleans)
    /// without any prose or table formatting, etc.
    ///
    /// See https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain#_plumbing_porcelain
    fn is_porcelain(&self) -> bool {
        true
    }

    /// Returns the name of this command.
    ///
    /// This value is used for dispatch (that is: mapping the name provided on the command line
    /// to the appropriate command) and for help messages.
    fn name(&self) -> &'static str;

    /// A brief message describing this command's functionality.
    fn about(&self) -> &'static str;

    /// A long message describing this command's functionality.
    fn long_about(&self) -> Option<&'static str> {
        None
    }

    /// Initializes a [`ClapCommand`] representing this command and its subcommands (if any).
    ///
    /// Commands wishing to customize their `ClapCommand`'s arguments should override
    /// [`Self::configure_args`].
    fn clap_command(&self) -> ClapCommand {
        // Configure a 'base' clap configuration that has the command's name, about message,
        // version, and author.

        let mut base_command = ClapCommand::new(self.name())
            .about(self.about())
            .version(crate_version!())
            .author(crate_authors!())
            .with_decompression_control()
            .arg(
                Arg::new(UNSTABLE_FLAG)
                    .short('X')
                    .long("unstable")
                    .default_value("false")
                    .action(ArgAction::SetTrue)
                    .value_parser(ValueParser::bool())
                    .help("Opt in to using an unstable feature of Ion CLI.")
                    .display_order(usize::MAX)
                    .hide(true),
            );

        if let Some(long_about) = self.long_about() {
            base_command = base_command.long_about(long_about)
        }

        if !self.is_stable() {
            let about = base_command.get_about().map(|x| x.to_string());
            if about.is_some() {
                base_command = base_command.about(format!("(UNSTABLE) {}", about.unwrap()))
            }
            base_command = base_command
                .before_help("WARNING: This command is unstable and requires explicit opt-in using '--unstable' or '-X'.");
        }
        if self.is_porcelain() {
            base_command = base_command.after_help(
                "NOTE: The output of this command is not intended to be machine-readable.",
            );
        }

        self.configure_args(base_command)
    }

    /// After initializing a [`ClapCommand`] using [Self::clap_command], subcommands may customize
    /// the `ClapCommand` further by overriding this default implementation.
    fn configure_args(&self, command: ClapCommand) -> ClapCommand {
        // Does nothing by default
        command
    }

    /// The core logic of the command.
    ///
    /// The default implementation assumes this command is a namespace (i.e. a group of subcommands).
    /// It looks for a subcommand in the arguments, then looks up and runs that subcommand.
    ///
    /// Commands should override this implementation.
    fn run(&self, command_path: &mut Vec<String>, args: &ArgMatches) -> anyhow::Result<()>;
}

/// Argument ID for the '--unstable' / '-X' flag
const UNSTABLE_FLAG: &str = "unstable";
/// Argument ID for the '--ion-version' / '-v' flag
const ION_VERSION_ARG_ID: &str = "ion-version";

/// Extension methods for a [`ClapCommand`] which add flags and options that are common to
/// commands in the Ion CLI.
pub trait WithIonCliArgument {
    fn with_input(self) -> Self;
    fn with_output(self) -> Self;
    fn with_format(self) -> Self;
    fn with_ion_version(self) -> Self;
    fn with_decompression_control(self) -> Self;
    fn show_unstable_flag(self) -> Self;
}

impl WithIonCliArgument for ClapCommand {
    fn with_input(self) -> Self {
        self.arg(
            Arg::new("input")
                .trailing_var_arg(true)
                .action(ArgAction::Append)
                .help("Input file"),
        )
    }

    fn with_output(self) -> Self {
        self.arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .help("Output file [default: STDOUT]"),
        )
    }

    fn with_format(self) -> Self {
        self.arg(
            Arg::new("format")
                .long("format")
                .short('f')
                .default_value("pretty")
                .value_parser(["binary", "text", "pretty", "lines"])
                .help("Output format"),
        )
    }

    fn with_ion_version(self) -> Self {
        // TODO When this arg/feature becomes stable:
        //    Remove: show_unstable_flag()
        //    Remove: requires(USE_UNSTABLE_FLAG)
        //    Add:    env("ION_CLI_ION_VERSION")
        self.show_unstable_flag()
            .arg(
                Arg::new(ION_VERSION_ARG_ID)
                    .long("ion-version")
                    // TODO: Should we find a different short version so that 'v' can be reserved
                    //       for 'verbose' in future use?
                    .short('v')
                    .help("UNSTABLE! Output Ion version")
                    .value_parser(["1.0", "1.1"])
                    .default_value("1.0")
                    .requires(UNSTABLE_FLAG),
            )
            .mut_arg(UNSTABLE_FLAG, |a| a.hide(false))
    }

    fn with_decompression_control(self) -> Self {
        let accepts_input = self
            .get_arguments()
            .any(|flag| dbg!(dbg!(flag.get_id()) == "input"));
        self.arg(
            Arg::new("no-auto-decompress")
                .long("no-auto-decompress")
                .default_value("false")
                .action(ArgAction::SetTrue)
                .help("Turn off automatic decompression detection.")
                // Do not show this flag in `help` for commands that don't take an `--input` flag.
                .hide(!accepts_input),
        )
    }

    /// All commands automatically have the "unstable" opt-in flag. This makes it visible.
    fn show_unstable_flag(self) -> Self {
        self.mut_arg(UNSTABLE_FLAG, |arg| arg.hide(false))
    }
}

/// Offers convenience methods for working with input and output streams.
pub struct CommandIo<'a> {
    args: &'a ArgMatches,
}

impl CommandIo<'_> {
    fn new(args: &ArgMatches) -> CommandIo {
        CommandIo { args }
    }

    /// Returns `true` if the user has not explicitly disabled auto decompression.
    fn auto_decompression_enabled(&self) -> bool {
        if let Some(is_disabled) = self.args.get_one::<bool>("no-auto-decompress") {
            !*is_disabled
        } else {
            true
        }
    }

    /// Constructs a new [`CommandInput`] representing STDIN.
    fn command_input_for_stdin(&self) -> Result<CommandInput> {
        const STDIN_NAME: &str = "-";
        let stdin = std::io::stdin().lock();
        if self.auto_decompression_enabled() {
            CommandInput::decompress(STDIN_NAME, stdin)
        } else {
            CommandInput::without_decompression(STDIN_NAME, stdin)
        }
    }

    /// Constructs a new [`CommandInput`] representing the specified file.
    fn command_input_for_file_name(&self, name: &str) -> Result<CommandInput> {
        let stream = File::open(name)?;
        if self.auto_decompression_enabled() {
            CommandInput::decompress(name, stream)
        } else {
            CommandInput::without_decompression(name, stream)
        }
    }

    /// Calls the provided closure once for each input source specified by the user.
    /// For each invocation, provides a handle to the configured output stream.
    fn for_each_input(
        &mut self,
        mut f: impl FnMut(&mut CommandOutput, CommandInput) -> Result<()>,
    ) -> Result<()> {
        // These types are provided by the `termcolor` crate. They wrap the normal `io::Stdout` and
        // `io::StdOutLock` types, making it possible to write colorful text to the output stream when
        // it's a TTY that understands formatting escape codes. These variables are declared here so
        // the lifetime will extend through the remainder of the function. Unlike `io::StdoutLock`,
        // the `StandardStreamLock` does not have a static lifetime.
        let stdout: StandardStream;
        let stdout_lock: StandardStreamLock;
        let mut output = if let Some(output_file) = self.args.get_one::<String>("output") {
            // If the user has specified an output file, use it.
            let file = File::create(output_file).with_context(|| {
                format!(
                    "could not open file output file '{}' for writing",
                    output_file
                )
            })?;
            CommandOutput::File(FileWriter::new(file))
        } else {
            // Otherwise, write to STDOUT.
            stdout = StandardStream::stdout(ColorChoice::Always);
            stdout_lock = stdout.lock();
            CommandOutput::StdOut(stdout_lock)
        };
        if let Some(input_file_names) = self.args.get_many::<String>("input") {
            // Input files were specified, run the converter on each of them in turn
            for input_file_name in input_file_names {
                let input = self.command_input_for_file_name(input_file_name)?;
                f(&mut output, input)?;
            }
        } else {
            let input = self.command_input_for_stdin()?;
            f(&mut output, input)?;
        }
        output.flush()?;
        Ok(())
    }

    fn write_output(&self, mut f: impl FnMut(&mut CommandOutput) -> Result<()>) -> Result<()> {
        // These types are provided by the `termcolor` crate. They wrap the normal `io::Stdout` and
        // `io::StdOutLock` types, making it possible to write colorful text to the output stream when
        // it's a TTY that understands formatting escape codes. These variables are declared here so
        // the lifetime will extend through the remainder of the function. Unlike `io::StdoutLock`,
        // the `StandardStreamLock` does not have a static lifetime.
        let stdout: StandardStream;
        let stdout_lock: StandardStreamLock;
        let mut output = if let Some(output_file) = self.args.get_one::<String>("output") {
            // If the user has specified an output file, use it.
            let file = File::create(output_file).with_context(|| {
                format!(
                    "could not open file output file '{}' for writing",
                    output_file
                )
            })?;
            CommandOutput::File(FileWriter::new(file))
        } else {
            // Otherwise, write to STDOUT.
            stdout = StandardStream::stdout(ColorChoice::Always);
            stdout_lock = stdout.lock();
            CommandOutput::StdOut(stdout_lock)
        };
        f(&mut output)
    }
}
