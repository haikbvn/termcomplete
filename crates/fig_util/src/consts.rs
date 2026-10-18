pub const APP_BUNDLE_ID: &str = "io.github.haikbvn.termcomplete";
pub const APP_BUNDLE_NAME: &str = "Termcomplete.app";

#[cfg(target_os = "macos")]
pub const APP_PROCESS_NAME: &str = "tcomp-desktop";
#[cfg(target_os = "linux")]
pub const APP_PROCESS_NAME: &str = "tcomp-desktop";

#[cfg(windows)]
pub const APP_PROCESS_NAME: &str = "q_desktop.exe";

/// The name configured under `"package.productName"` in the tauri.conf.json file.
pub const TAURI_PRODUCT_NAME: &str = "tcomp-desktop";

pub const CLI_BINARY_NAME: &str = "tcomp";
pub const CLI_BINARY_NAME_MINIMAL: &str = "q-minimal";
pub const PTY_BINARY_NAME: &str = "tcomp-term";

pub const CLI_CRATE_NAME: &str = "q_cli";

pub const URL_SCHEMA: &str = "tcomp";

pub const PRODUCT_NAME: &str = "Termcomplete";

pub const RUNTIME_DIR_NAME: &str = "tcomp";

/// Data directory name used in paths like ~/.local/share/{DATA_DIR_NAME}
#[cfg(unix)]
pub const DATA_DIR_NAME: &str = "termcomplete";
#[cfg(windows)]
pub const DATA_DIR_NAME: &str = "AmazonQ";

/// Backup directory name
pub const BACKUP_DIR_NAME: &str = ".termcomplete.dotfiles.bak";

// These are the old "CodeWhisperer" branding, used anywhere we will not update to Termcomplete
pub const OLD_PRODUCT_NAME: &str = "CodeWhisperer";
pub const OLD_CLI_BINARY_NAMES: &[&str] = &["cw"];
pub const OLD_PTY_BINARY_NAMES: &[&str] = &["cwterm"];

pub const GITHUB_REPO_NAME: &str = "haikbvn/termcomplete";

pub mod url {
    pub const USER_MANUAL: &str = "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line.html";
    pub const AUTOCOMPLETE_WIKI: &str =
        "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-autocomplete.html";
    pub const AUTOCOMPLETE_SSH_WIKI: &str =
        "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-autocomplete-ssh.html";
    pub const CHAT_WIKI: &str = "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-chat.html";
    pub const TRANSLATE_WIKI: &str =
        "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-conversation.html";
    pub const TELEMETRY_WIKI: &str = "https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/opt-out-IDE.html";
}

/// Build time env vars
pub mod build {
    /// The target of the current build, e.g. "aarch64-unknown-linux-musl"
    pub const TARGET_TRIPLE: Option<&str> = option_env!("TCOMP_BUILD_TARGET_TRIPLE");

    /// The variant of the current build
    pub const VARIANT: Option<&str> = option_env!("TCOMP_BUILD_VARIANT");

    /// A git full sha hash of the current build
    pub const HASH: Option<&str> = option_env!("TCOMP_BUILD_HASH");

    /// The datetime in rfc3339 format of the current build
    pub const DATETIME: Option<&str> = option_env!("TCOMP_BUILD_DATETIME");

    /// If `fish` tests should be skipped
    pub const SKIP_FISH_TESTS: bool = option_env!("TCOMP_BUILD_SKIP_FISH_TESTS").is_some();

    /// If `shellcheck` tests should be skipped
    pub const SKIP_SHELLCHECK_TESTS: bool = option_env!("TCOMP_BUILD_SKIP_SHELLCHECK_TESTS").is_some();
}

/// macOS specific constants
pub mod macos {
    pub const BUNDLE_CONTENTS_MACOS_PATH: &str = "Contents/MacOS";
    pub const BUNDLE_CONTENTS_RESOURCE_PATH: &str = "Contents/Resources";
    pub const BUNDLE_CONTENTS_HELPERS_PATH: &str = "Contents/Helpers";
    pub const BUNDLE_CONTENTS_INFO_PLIST_PATH: &str = "Contents/Info.plist";
}

pub mod linux {
    pub const DESKTOP_ENTRY_NAME: &str = "amazon-q.desktop";

    /// Name of the deb package.
    pub const PACKAGE_NAME: &str = "termcomplete";

    /// The wm_class used for the application windows.
    pub const DESKTOP_APP_WM_CLASS: &str = "Amazon-q";
}

pub mod env_var {
    macro_rules! define_env_vars {
        ($($(#[$meta:meta])* $ident:ident = $name:expr),*) => {
            $(
                $(#[$meta])*
                pub const $ident: &str = $name;
            )*

            pub const ALL: &[&str] = &[$($ident),*];
        }
    }

    define_env_vars! {
        /// The UUID of the current parent qterm instance
        TCOMP_PTY_SESSION_ID = "TCOMP_PTY_SESSION_ID",

        /// The current parent socket to connect to
        TCOMP_PARENT = "TCOMP_PARENT",

        /// Set the [`TCOMP_PARENT`] parent socket to connect to
        TCOMP_SET_PARENT = "TCOMP_SET_PARENT",

        /// Guard for the [`TCOMP_SET_PARENT`] check
        TCOMP_SET_PARENT_CHECK = "TCOMP_SET_PARENT_CHECK",

        /// Set if qterm is running, contains the version
        TCOMP_TERM = "TCOMP_TERM",

        /// Sets the current log level
        TCOMP_LOG_LEVEL = "TCOMP_LOG_LEVEL",

        /// Overrides the ZDOTDIR environment variable
        TCOMP_ZDOTDIR = "TCOMP_ZDOTDIR",

        /// Indicates a process was launched by Termcomplete
        PROCESS_LAUNCHED_BY_TCOMP = "PROCESS_LAUNCHED_BY_TCOMP",

        /// The shell to use in qterm
        TCOMP_SHELL = "TCOMP_SHELL",

        /// Indicates the user is debugging the shell
        TCOMP_DEBUG_SHELL = "TCOMP_DEBUG_SHELL",

        /// Indicates the user is using zsh autosuggestions which disables Inline
        TCOMP_USING_ZSH_AUTOSUGGESTIONS = "TCOMP_USING_ZSH_AUTOSUGGESTIONS",

        /// Overrides the path to the bundle metadata released with certain desktop builds.
        TCOMP_BUNDLE_METADATA_PATH = "TCOMP_BUNDLE_METADATA_PATH"
    }
}

pub mod system_paths {
    /// System installation paths
    pub const APPLICATIONS_DIR: &str = "/Applications";
    pub const USR_LOCAL_BIN: &str = "/usr/local/bin";
    pub const USR_SHARE: &str = "/usr/share";
    pub const OPT_HOMEBREW_BIN: &str = "/opt/homebrew/bin";
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    use super::*;

    #[test]
    fn test_build_envs() {
        if let Some(build_variant) = build::VARIANT {
            println!("build_variant: {build_variant}");
            assert!(["full", "minimal"].contains(&&*build_variant.to_ascii_lowercase()));
        }

        if let Some(build_hash) = build::HASH {
            println!("build_hash: {build_hash}");
            assert!(!build_hash.is_empty());
        }

        if let Some(build_datetime) = build::DATETIME {
            println!("build_datetime: {build_datetime}");
            println!("{}", OffsetDateTime::parse(build_datetime, &Rfc3339).unwrap());
        }
    }
}
