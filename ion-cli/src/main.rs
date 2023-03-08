use std::io;
use std::io::{stdin, stdout};
use anyhow::Result;
use ion_cli_core::execute;
use ion_cli_core::fs_wrapper::RealFileSystem;

fn main() -> Result<()> {
    execute(|app| app.get_matches(), &mut stdin().lock(), &mut stdout().lock(), RealFileSystem)
}
