use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::NamedTempFile;

fn veiled() -> Command {
    cargo_bin_cmd!("veiled")
}

// -- help and version --

#[test]
fn help_displays_all_commands() {
    veiled()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("reset"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("update"));
}

#[test]
fn version_displays_cargo_version() {
    veiled()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

// -- add command --

#[test]
fn add_nonexistent_path_exits_with_error() {
    veiled()
        .args(["add", "/nonexistent/path/that/does/not/exist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn add_file_instead_of_directory_exits_with_error() {
    let file = NamedTempFile::new().unwrap();

    veiled()
        .args(["add", file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a directory"));
}

#[test]
fn add_requires_path_argument() {
    veiled().arg("add").assert().failure();
}

#[test]
fn add_help_shows_path_argument() {
    veiled()
        .args(["add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<PATH>").or(predicate::str::contains("path")));
}

// -- list command --

#[test]
fn list_exits_successfully() {
    veiled().arg("list").assert().success();
}

// -- status command --

#[test]
fn status_shows_daemon_state() {
    veiled()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Daemon:"));
}

#[test]
fn status_shows_exclusion_info() {
    veiled().arg("status").assert().success().stdout(
        predicate::str::contains("excluded by veiled")
            .or(predicate::str::contains("No exclusions")),
    );
}

// -- reset command --

#[test]
fn reset_aborts_on_decline() {
    veiled().arg("reset").write_stdin("n\n").assert().success();
}

#[test]
fn reset_help_shows_yes_flag() {
    veiled()
        .args(["reset", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--yes"));
}

// -- start command --

#[test]
fn start_help_shows_description() {
    veiled()
        .args(["start", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// -- stop command --

#[test]
fn stop_without_daemon_prints_message() {
    veiled()
        .arg("stop")
        .assert()
        .success()
        .stdout(predicate::str::contains("not running"));
}

// -- update command --

#[test]
fn update_help_shows_description() {
    veiled()
        .args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Update binary to the latest version",
        ));
}

#[test]
fn update_displays_current_version() {
    // update will fail (no releases / network) but should print the current version first
    veiled()
        .arg("update")
        .assert()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

// -- unknown command --

#[test]
fn unknown_command_exits_with_error() {
    veiled().arg("foobar").assert().failure();
}
