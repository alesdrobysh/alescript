use assert_cmd::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_hello_world() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo_bin!("alescript"));
    cmd.arg("examples/hello.ales");
    cmd.assert()
        .success()
        .stdout(predicate::eq("hello, world!\n"));
    Ok(())
}

#[test]
fn test_fibonacci() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo_bin!("alescript"));
    cmd.arg("examples/fibonacci.ales");
    cmd.assert().success().stdout(predicate::eq("55% ABV\n"));
    Ok(())
}

#[test]
fn test_brewing() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo_bin!("alescript"));
    cmd.arg("examples/brewing.ales");

    // Using contains because of potential floating point precision differences across platforms
    // although they should be consistent for these simple calculations.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("7% ABV"))
        .stdout(predicate::str::contains("8% ABV"))
        .stdout(predicate::str::contains("strong brew!"))
        .stdout(predicate::str::contains("29.42857142857143% ABV"))
        .stdout(predicate::str::contains("19.42857142857143% ABV"));
    Ok(())
}

#[test]
fn test_friday() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo_bin!("alescript"));
    cmd.arg("examples/friday.ales");

    // Note: friday.ales has a judge statement that might be flaky due to fuzzy logic.
    // However, 5.5% vs 6.0% is usually enough of a gap.
    let assert = cmd.assert().success();

    assert
        .stdout(predicate::str::contains(
            "Work week ending... Brewing in progress.",
        ))
        .stdout(predicate::str::contains("Your pint is ready. Cheers!"));

    Ok(())
}

#[test]
fn test_fixes() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo_bin!("alescript"));
    cmd.arg("examples/test_fixes.ales");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test 1 passed"))
        .stdout(predicate::str::contains("After fortifying by 3"));
    Ok(())
}

#[test]
fn test_full_syntax() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo_bin!("alescript"));
    cmd.arg("examples/full-syntax-example.ales");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("10.274999999999999% ABV"));
    Ok(())
}
