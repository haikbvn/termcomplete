use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn version_flag_has_status_code_zero() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tcomp-term"));
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
