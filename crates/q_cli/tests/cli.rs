use std::io::Write;
use std::process::{Command, Stdio};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tcomp"))
}

#[test]
fn standalone_command_contract() {
    let help = cli().arg("--help").output().unwrap();
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("Standalone terminal autocomplete"));
    for command in ["install", "uninstall", "doctor", "settings", "theme"] {
        assert!(text.contains(command));
    }
    for command in ["login", "chat", "translate", "agent"] {
        assert!(!cli().args([command, "--help"]).status().unwrap().success());
    }
    let version = cli().arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        format!("tcomp {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn settings_keep_json_types_and_unrelated_keys() {
    let directory = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        let result = cli()
            .env("TCOMP_DATA_DIR", directory.path())
            .args(args)
            .output()
            .unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        result.stdout
    };
    run(&["settings", "autocomplete.disable", "true"]);
    run(&["settings", "appearance.theme", "dark"]);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&run(&["settings", "autocomplete.disable"])).unwrap(),
        true
    );
    run(&["settings", "autocomplete.disable", "--delete"]);
    let settings: serde_json::Value = serde_json::from_slice(&run(&["settings"])).unwrap();
    assert_eq!(settings["appearance.theme"], "dark");
    assert!(settings.get("autocomplete.disable").is_none());
}

#[test]
fn generated_bash_and_zsh_parse_without_login_or_ai_hooks() {
    for (shell, exe) in [("bash", "/bin/bash"), ("zsh", "/bin/zsh")] {
        for when in ["pre", "post"] {
            let result = cli()
                .env("TCOMP_INIT_SNAPSHOT_TEST", "1")
                .args(["init", shell, when])
                .output()
                .unwrap();
            assert!(result.status.success());
            let text = String::from_utf8(result.stdout).unwrap();
            assert!(
                !text.contains("tcomp login")
                    && !text.contains("tcomp inline")
                    && !text.contains("QTERM_SESSION_ID")
                    && !text.contains("fig_osc")
                    && !text.contains("__fig_")
            );
            let mut parser = Command::new(exe).arg("-n").stdin(Stdio::piped()).spawn().unwrap();
            parser.stdin.take().unwrap().write_all(text.as_bytes()).unwrap();
            assert!(parser.wait().unwrap().success(), "{shell} {when}");
        }
    }
}
