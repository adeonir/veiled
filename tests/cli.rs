use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::{NamedTempFile, TempDir};

fn veiled() -> (Command, TempDir) {
    let dir = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("veiled");
    cmd.env("VEILED_CONFIG_DIR", dir.path());
    (cmd, dir)
}

// -- help and version --

#[test]
fn help_displays_all_commands() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("reset"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("update"));
}

#[test]
fn help_shows_package_description() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Time Machine"));
}

#[test]
fn version_displays_cargo_version() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

// -- add command --

#[test]
fn add_nonexistent_path_exits_with_error() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["add", "/nonexistent/path/that/does/not/exist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn add_file_instead_of_directory_exits_with_error() {
    let file = NamedTempFile::new().unwrap();
    let (mut cmd, _dir) = veiled();
    cmd.args(["add", file.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a directory"));
}

#[test]
fn add_requires_path_argument() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("add").assert().failure();
}

#[test]
fn add_help_shows_path_argument() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<PATH>").or(predicate::str::contains("path")));
}

#[test]
fn add_warns_on_nested_path() {
    let parent = TempDir::new().unwrap();
    let child = parent.path().join("nested");
    std::fs::create_dir(&child).unwrap();

    let (mut cmd1, dir) = veiled();
    cmd1.args(["add", parent.path().to_str().unwrap()])
        .assert()
        .success();

    let mut cmd2 = cargo_bin_cmd!("veiled");
    cmd2.env("VEILED_CONFIG_DIR", dir.path());
    cmd2.args(["add", child.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("already covered by"));
}

// -- remove command --

#[test]
fn remove_unmanaged_path_shows_not_managed() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["remove", "/nonexistent/path/that/does/not/exist"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not managed by veiled"));
}

#[test]
fn remove_requires_path_argument() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("remove").assert().failure();
}

#[test]
fn remove_help_shows_path_argument() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<PATH>").or(predicate::str::contains("path")));
}

// -- list command --

#[test]
fn list_exits_successfully() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("list").assert().success();
}

// -- status command --

#[test]
fn status_shows_daemon_state() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Daemon:"));
}

#[test]
fn status_shows_exclusion_info() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("status").assert().success().stdout(
        predicate::str::contains("excluded by veiled")
            .or(predicate::str::contains("No exclusions")),
    );
}

#[test]
fn status_refresh_flag_accepted() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["status", "--refresh"]).assert().success();
}

#[test]
fn status_help_shows_refresh_flag() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--refresh"));
}

// -- reset command --

#[test]
fn reset_aborts_on_decline() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("reset").write_stdin("n\n").assert().success();
}

#[test]
fn reset_help_shows_yes_flag() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["reset", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--yes"));
}

// -- start command --

#[test]
fn start_help_shows_description() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["start", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// -- stop command --

#[test]
fn stop_help_shows_description() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["stop", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deactivate daemon"));
}

// -- update command --

#[test]
fn update_help_shows_description() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Update binary to the latest version",
        ));
}

#[test]
fn update_displays_current_version() {
    // update will fail (no releases / network) but should print the current version first
    let (mut cmd, _dir) = veiled();
    cmd.arg("update")
        .assert()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

// -- verbose flag --

#[test]
fn verbose_flag_accepted_before_subcommand() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["--verbose", "list"]).assert().success();
}

#[test]
fn verbose_flag_accepted_with_status() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["--verbose", "status"]).assert().success();
}

#[test]
fn verbose_flag_shown_in_help() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--verbose"));
}

// -- FDA warning --

#[test]
fn status_fda_warning_on_stderr_if_present() {
    let (mut cmd, _dir) = veiled();
    let output = cmd.args(["status"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // FDA check may or may not fail depending on environment;
    // if there is stderr output, it must contain the expected warning text
    if !stderr.is_empty() {
        assert!(
            stderr.contains("Full Disk Access may be required"),
            "unexpected stderr: {stderr}"
        );
    }
}

#[test]
fn verbose_status_shows_fda_detail_if_warning() {
    let (mut cmd, _dir) = veiled();
    let output = cmd.args(["--verbose", "status"]).output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // When verbose + FDA warning, both "warning:" and "detail:" lines should appear
    if stderr.contains("Full Disk Access may be required") {
        assert!(
            stderr.contains("detail:"),
            "verbose mode should include detail line: {stderr}"
        );
    }
}

#[test]
fn start_help_shows_install_description() {
    let (mut cmd, _dir) = veiled();
    cmd.args(["start", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Install binary"));
}

// -- unknown command --

#[test]
fn unknown_command_exits_with_error() {
    let (mut cmd, _dir) = veiled();
    cmd.arg("foobar").assert().failure();
}
