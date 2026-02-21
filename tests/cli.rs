use assert_cmd::cargo::cargo_bin_cmd;

fn veiled() -> assert_cmd::Command {
    cargo_bin_cmd!("veiled")
}

#[test]
fn help_displays_all_commands() {
    let output = veiled().arg("--help").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    for cmd in [
        "start", "stop", "run", "list", "reset", "add", "status", "update",
    ] {
        assert!(stdout.contains(cmd), "--help output missing command: {cmd}");
    }
}

#[test]
fn version_displays_cargo_version() {
    let output = veiled().arg("--version").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}
