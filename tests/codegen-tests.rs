use std::env;
use std::io;
use std::io::Write;
use std::fmt::Display;
use anyhow::Result;

#[cfg(feature = "beta-subcommands")]
#[test]
// Todo: Parameterize using rstest iff we have multiple rust projects in our codegen tests directory.
fn test_codegen_cargo_project() -> Result<()> {
    // Get absolute path for the things we need
    let cargo_executable = env!("CARGO");
    let ion_executable = env!("CARGO_BIN_EXE_ion");
    let test_crate_path = format!("{}/codegen-test-projects/rust", env!("CARGO_MANIFEST_DIR"));

    // Clean
    let cargo_clean_output = std::process::Command::new(cargo_executable)
        .current_dir(&test_crate_path)
        .arg("clean")
        .output()
        .expect("failed to execute 'cargo clean'");

    if !cargo_clean_output.status.success() {
        println!("status: {}", cargo_clean_output.status);
        io::stdout().write_all(&cargo_clean_output.stdout).unwrap();
        io::stderr().write_all(&cargo_clean_output.stderr).unwrap();
        assert!(cargo_clean_output.status.success());
    } else {
        io::stdout().write_all(&cargo_clean_output.stdout).unwrap();
    }

    // Test
    let cargo_test_output = std::process::Command::new(cargo_executable)
        .current_dir(&test_crate_path)
        .env("ION_CLI", ion_executable)
        .arg("test")
        .arg("--").arg("--nocapture")
        .output()
        .expect("failed to execute 'cargo test'");

    println!("status: {}", cargo_test_output.status);
    io::stdout().write_all(&cargo_test_output.stdout).unwrap();
    io::stderr().write_all(&cargo_test_output.stderr).unwrap();

    assert!(cargo_test_output.status.success());
    Ok(())
}

#[cfg(feature = "beta-subcommands")]
#[test]
// Todo: Parameterize using rstest iff we have multiple gradle projects in our codegen tests directory.
fn test_codegen_gradle_project() -> Result<()> {
    // Get absolute path for the things we need
    let ion_executable = env!("CARGO_BIN_EXE_ion");
    let test_crate_path = format!("{}/codegen-test-projects/java", env!("CARGO_MANIFEST_DIR"));
    let gradle_executable = format!("{}/gradlew", test_crate_path);

    // Clean
    let gradle_output = std::process::Command::new(gradle_executable)
        .current_dir(&test_crate_path)
        .env("ION_CLI", ion_executable)
        .arg("clean")
        .arg("build")
        .output()
        .expect("failed to execute './gradlew clean build'");

    println!("status: {}", gradle_output.status);
    io::stdout().write_all(&gradle_output.stdout).unwrap();
    io::stderr().write_all(&gradle_output.stderr).unwrap();

    assert!(gradle_output.status.success());
    Ok(())
}

