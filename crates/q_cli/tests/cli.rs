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
    for command in ["install", "uninstall"] {
        let result = cli()
            .args([command, "--shell-only", "--input-method-only"])
            .output()
            .unwrap();
        assert!(!result.status.success(), "Conflicting install scopes must be rejected");
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
fn cursor_helper_activation_errors_are_structured() {
    for enable in [false, true] {
        let mut command = cli();
        command.args([
            "internal",
            "attempt-to-finish-input-method-installation",
            "relative.app",
        ]);
        if enable {
            command.arg("--enable");
        }
        let result = command.output().unwrap();
        assert!(!result.status.success());
        let error: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(error, "InvalidBundlePath");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn helper_only_requests_the_desktop_without_editing_shell_files() {
    use std::os::unix::fs::PermissionsExt;
    use fig_ipc::{BufferedUnixStream, RecvMessage, SendMessage};
    use fig_proto::local::{self, command, command_response, local_message};
    let root = tempfile::Builder::new()
        .prefix("tcomp-ipc-")
        .tempdir_in("/private/tmp")
        .unwrap();
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let shell = root.path().join("shell");
    std::fs::create_dir(&shell).unwrap();
    std::fs::write(shell.join(".zshrc"), "# preserve this shell configuration\n").unwrap();
    let listener = tokio::net::UnixListener::bind(root.path().join("desktop.sock")).unwrap();
    let server = tokio::spawn(async move {
        for action in [local::InputMethodAction::Install, local::InputMethodAction::Uninstall] {
            let (socket, _) = listener.accept().await.unwrap();
            let mut socket = BufferedUnixStream::new(socket);
            let message: local::LocalMessage = socket.recv_message().await.unwrap().unwrap();
            let Some(local_message::Type::Command(request)) = message.r#type else {
                panic!("Expected command")
            };
            let Some(command::Command::InputMethod(helper)) = request.command else {
                panic!("Expected helper request")
            };
            assert_eq!(helper.actions(), action);
            socket
                .send_message(local::CommandResponse {
                    id: request.id,
                    response: Some(command_response::Response::Success(local::SuccessResponse {
                        message: None,
                    })),
                })
                .await
                .unwrap();
        }
    });
    for action in ["install", "uninstall"] {
        let output = cli()
            .args([action, "--input-method-only"])
            .env("TCOMP_RUNTIME_DIR", root.path())
            .env("TCOMP_DATA_DIR", root.path().join("data"))
            .env("TCOMP_SHELL_CONFIG_DIR", &shell)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
    server.await.unwrap();
    assert_eq!(
        std::fs::read_to_string(shell.join(".zshrc")).unwrap(),
        "# preserve this shell configuration\n"
    );
    assert!(!shell.join(".bashrc").exists());
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
